#!/usr/bin/env python3
"""Reference answers for Rust from rust-analyzer over LSP `textDocument/references`.

    truth_rust.py <workspace> <source-root>   (targets as JSON on stdin)

`workspace` is the full checkout (rust-analyzer needs `Cargo.toml` and the dependencies), `source-root`
is the filtered copy that CodeSkimr scanned. Targets carry paths relative to both. Prints per target the
list of [file, line] of the uses that lie inside the filtered copy. This does not read the SCIP index
that CodeSkimr uses for level 2: the answers come from the language server alone (ADR 0033, decision 14).
"""
import json, os, re, subprocess, sys, time
from pathlib import Path

START_TIMEOUT_SECONDS = 1800
REQUEST_TIMEOUT_SECONDS = 120
LOOKAHEAD_LINES = 8


class Lsp:
    def __init__(self, root):
        self.proc = subprocess.Popen(["rust-analyzer"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                     stderr=subprocess.DEVNULL, cwd=root)
        self.next_id, self.pending = 1, []
        self.quiescent = False

    def send(self, message):
        body = json.dumps({"jsonrpc": "2.0", **message}).encode()
        self.proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
        self.proc.stdin.flush()

    def read(self):
        length = 0
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise EOFError("rust-analyzer closed the connection")
            if line in (b"\r\n", b"\n"):
                break
            if line.lower().startswith(b"content-length:"):
                length = int(line.split(b":")[1])
        return json.loads(self.proc.stdout.read(length))

    def handle(self, message):
        """Answer requests of the server and note its status; returns the message if it is a response."""
        if "method" in message and "id" in message:
            result = [{}] if message["method"] == "workspace/configuration" else None
            self.send({"id": message["id"], "result": result})
        elif message.get("method") == "experimental/serverStatus":
            self.quiescent = bool(message["params"].get("quiescent")) and message["params"].get("health") != "error"
        return message if "method" not in message else None

    def request(self, method, params):
        request_id = self.next_id
        self.next_id += 1
        self.send({"id": request_id, "method": method, "params": params})
        while True:
            reply = self.handle(self.read())
            if reply is not None and reply.get("id") == request_id:
                return reply.get("result")

    def wait_quiescent(self):
        deadline = time.time() + START_TIMEOUT_SECONDS
        while not self.quiescent:
            if time.time() > deadline:
                raise TimeoutError("rust-analyzer did not finish loading")
            self.handle(self.read())


def name_position(lines, target):
    """Line and column of the declared name: the declaration may start above it (attributes, `pub`)."""
    pattern = re.compile(r"(?<![A-Za-z0-9_])" + re.escape(target["name"]) + r"(?![A-Za-z0-9_])")
    for index in range(target["line"] - 1, min(len(lines), target["line"] - 1 + LOOKAHEAD_LINES)):
        found = pattern.search(lines[index], target["column"] if index == target["line"] - 1 else 0)
        if found:
            return index, found.start()
    return None


def utf16(text, column):
    return len(text[:column].encode("utf-16-le")) // 2


def main():
    workspace, source = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    targets = json.load(sys.stdin)
    lsp = Lsp(workspace)
    lsp.send({"id": 0, "method": "initialize", "params": {
        "processId": os.getpid(), "rootUri": workspace.as_uri(),
        "capabilities": {"experimental": {"serverStatusNotification": True},
                         "window": {"workDoneProgress": True}},
        "initializationOptions": {"cargo": {"buildScripts": {"enable": False}}, "procMacro": {"enable": False},
                                  "checkOnSave": False, "files": {"watcher": "server"}}}})
    while lsp.read().get("id") != 0:
        pass
    lsp.send({"method": "initialized", "params": {}})
    lsp.wait_quiescent()
    out, opened = [], set()
    for target in targets:
        path = workspace / target["file"]
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        uses, position = set(), name_position(lines, target)
        if position is None:
            print(f"ra: {target['name']}: name not found", file=sys.stderr)
        else:
            if path not in opened:
                lsp.send({"method": "textDocument/didOpen", "params": {"textDocument": {
                    "uri": path.as_uri(), "languageId": "rust", "version": 1, "text": text}}})
                opened.add(path)
            line, column = position
            result = lsp.request("textDocument/references", {
                "textDocument": {"uri": path.as_uri()},
                "position": {"line": line, "character": utf16(lines[line], column)},
                "context": {"includeDeclaration": False}}) or []
            for ref in result:
                file = Path(ref["uri"].removeprefix("file://")).resolve()
                if file.is_relative_to(workspace) and (source / file.relative_to(workspace)).exists():
                    uses.add((str(file.relative_to(workspace)).replace("\\", "/"), ref["range"]["start"]["line"] + 1))
        out.append(sorted(uses))
    print(json.dumps(out))
    lsp.proc.kill()


main()
