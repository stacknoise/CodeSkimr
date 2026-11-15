// Checks the M6 functions of blueprint 19 that need no keystore and no network (WP-76):
//   Git source   clone, update by a new commit, other branch, missing branch (SRC-202)
//   Export       one page, no source folder, no data folder, no planted secret
//   Secret       a canary planted in a method body and in the environment of the commands is nowhere in
//                their output, in the export or in the database and logs of the data folder
//   Keystore     what `credential set` says on a machine without a keystore (SEC-601, ADR 0045)
// It uses the release binary and a throw-away Git repository below the temp folder. Nothing is
// written outside it. The result is plain text on the console; paste it into the report.
//
// Usage, from the CodeSkimr root after `cargo build --release`:
//   node tools/acceptance/m6-measure.mjs [output-file]
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";

const root = path.resolve(import.meta.dirname, "../..");
const binary = path.join(root, "target/release", process.platform === "win32" ? "codeskimr.exe" : "codeskimr");
const CANARY = "CANARY-m6-5e2c9a71";
const work = mkdtempSync(path.join(tmpdir(), "codeskimr-m6-"));
const data = path.join(work, "data");
const lines = [];
const outputs = [];

/** Writes a line of the result; the paths of this machine are replaced so the file can be committed. */
function log(text) {
    const neutral = text.split(work).join("<tmp>").split(root).join("<repo>");
    console.log(neutral);
    lines.push(neutral);
}

function run(program, args, options = {}) {
    const started = performance.now();
    const result = spawnSync(program, args, {
        encoding: "utf8",
        env: { ...process.env, CODESKIMR_LANG: "en", CODESKIMR_CANARY: CANARY },
        ...options,
    });
    const text = `${result.stdout ?? ""}${result.stderr ?? ""}`;
    outputs.push(text);
    return { status: result.status, text, ms: performance.now() - started };
}

const git = (cwd, ...args) => run("git", args, { cwd });
const app = (...args) => run(binary, ["--data-dir", data, ...args]);
const summary = (text) => /Scanned (\d+) files in ([\d.]+) s\.\n(\d+) symbols, (\d+) references/.exec(text);

function prepare() {
    const src = path.join(work, "src");
    mkdirSync(src);
    git(src, "init", "-q", "-b", "main");
    git(src, "config", "user.email", "m6@example.invalid");
    git(src, "config", "user.name", "m6");
    writeFileSync(path.join(src, "A.cs"), `public class A { public int F(int x) { var s = "${CANARY}"; return x + 1; } }\n`);
    git(src, "add", ".");
    git(src, "commit", "-qm", "one");
    git(work, "clone", "-q", "--bare", "src", "remote.git");
    return { src, url: `file://${path.join(work, "remote.git").split(path.sep).join("/")}` };
}

function push(src, branch, file, text) {
    git(src, "checkout", "-q", "-B", branch);
    writeFileSync(path.join(src, file), text);
    git(src, "add", ".");
    git(src, "commit", "-qm", file);
    git(src, "push", "-q", "../remote.git", branch);
}

function scanLine(label, result) {
    const found = summary(result.text);
    const facts = found ? `${found[1]} files, ${found[3]} symbols, ${found[4]} references, scan ${found[2]} s` : `exit ${result.status}`;
    log(`| ${label} | ${facts} | ${result.ms.toFixed(0)} ms |`);
}

function gitSource({ src, url }) {
    log("\n## Git source\n\n| Step | Result | Wall time |\n|---|---|---|");
    scanLine("clone and scan (first)", app("scan", url));
    scanLine("scan again, nothing new", app("scan", url));
    push(src, "main", "B.cs", "public class B { public void G() { new A().F(1); } }\n");
    scanLine("update after a new commit on main", app("scan", url));
    push(src, "dev", "C.cs", "public class C { }\n");
    scanLine("branch dev", app("scan", url, "--branch", "dev"));
    const missing = app("scan", url, "--branch", "nope");
    log(`| missing branch | exit ${missing.status}, ${/SRC-\d+/.exec(missing.text)?.[0] ?? "no code"} | ${missing.ms.toFixed(0)} ms |`);
}

function files(dir) {
    return readdirSync(dir, { withFileTypes: true, recursive: true })
        .filter((entry) => entry.isFile())
        .map((entry) => path.join(entry.parentPath, entry.name));
}

function count(texts, needle) {
    return texts.filter((text) => text.includes(needle)).length;
}

function exportPage({ url }) {
    log("\n## Export and secrets\n");
    const page = path.join(work, "export.html");
    const done = app("export", url, "--html", page);
    log(`- \`export --html\`: exit ${done.status}, ${done.text.trim().split("\n")[0]} (${done.ms.toFixed(0)} ms)`);
    const html = readFileSync(page, "utf8");
    log(`- page size: ${(statSync(page).size / 1024).toFixed(0)} KiB`);
    for (const [name, needle] of [["planted secret", CANARY], ["data folder", data], ["working folder", work]]) {
        log(`- occurrences of the ${name} in the page: ${html.split(needle).length - 1}`);
    }
    const inside = files(data).map((file) => [file, readFileSync(file, "latin1")]);
    const cache = inside.filter(([file]) => file.includes(`${path.sep}git${path.sep}`));
    const rest = inside.filter(([file]) => !cache.some(([other]) => other === file));
    log(`- files of the Git cache holding the planted text (the clone holds the source, expected): ${count(cache.map((e) => e[1]), CANARY)}`);
    log(`- other files of the data folder (database, logs, tokens) holding it: ${count(rest.map((e) => e[1]), CANARY)} of ${rest.length}`);
    log(`- outputs of all commands holding it: ${count(outputs, CANARY)} of ${outputs.length}`);
}

function keystore() {
    log("\n## Keystore\n");
    const set = run(binary, ["--data-dir", data, "credential", "set", "m6", "--user", "m6"], { input: `${CANARY}\n` });
    log(`- \`credential set\` here: exit ${set.status}, ${/(SEC|NET|CLI)-\d+/.exec(set.text)?.[0] ?? "stored"}`);
    log(`- the secret in its output: ${set.text.includes(CANARY) ? "yes" : "no"}`);
}

try {
    log(`# M6 measurement, ${new Date().toISOString().slice(0, 10)}, ${process.platform}, ${binary}`);
    const repo = prepare();
    gitSource(repo);
    exportPage(repo);
    keystore();
} finally {
    const target = process.argv[2];
    if (target) writeFileSync(target, `${lines.join("\n")}\n`);
    rmSync(work, { recursive: true, force: true });
}
