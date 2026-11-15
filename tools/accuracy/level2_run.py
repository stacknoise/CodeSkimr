"""Level 2 through the real way (ADR 0033, decision 14): `codeskimr level2 confirm`, then `codeskimr scan` with the
indexer started by the `IndexRunner`. Used by `measure.py --level2-run`; nothing here reads a SCIP index.

The corpus checkout is copied into a throw-away folder (indexers build in the project folder), prepared the way
`docs/accuracy.md` describes (build settings of the container, restored dependencies), and scanned with the
skipped folders of the corpus as `scan.excluded_folders`, so the scan sees the same files as the filtered copy the
reference tools read, while the indexer still sees the whole project.
"""
import json, os, pty, re, select, shutil, subprocess, sys, time
from pathlib import Path

CONFIRM_DELAY = 1.0
CONFIRM_TIMEOUT = 60


def prepare(lang, copy, tmp):
    """Build settings of the container and restored dependencies (what the user does before level 2). Returns extra environment."""
    env = {}
    if lang == "java":
        pom = copy / "pom.xml"
        text = re.sub(r"<failOnWarning>\s*true\s*</failOnWarning>", "<failOnWarning>false</failOnWarning>", pom.read_text())
        # Only the library module is scanned; the other modules need repositories the container cannot reach (test-shrinker: R8).
        pom.write_text(re.sub(r"<module>(?!gson<)[^<]*</module>\s*", "", text))
    elif lang == "csharp":
        for proj in copy.rglob("*.csproj"):
            text = proj.read_text()
            text = re.sub(r"<TargetFrameworks?>[^<]*</TargetFrameworks?>", "<TargetFrameworks>net8.0</TargetFrameworks>", text)
            proj.write_text(text)
    elif lang == "typescript":
        subprocess.run(["npm", "ci", "--ignore-scripts"], cwd=copy, check=True, capture_output=True)
    elif lang == "python":
        venv = tmp / "venv"
        subprocess.run([sys.executable, "-m", "venv", str(venv)], check=True)
        env = {"VIRTUAL_ENV": str(venv), "PATH_PREFIX": str(venv / "bin")}
    return env


def environment(extra, tools_bin):
    env = dict(os.environ)
    prefix = [extra.pop("PATH_PREFIX", None), str(tools_bin)]
    env["PATH"] = os.pathsep.join([p for p in prefix if p] + [env["PATH"]])
    env.update(extra)
    return env


def confirm(base, copy, env):
    """Types the project name at the prompt of `level2 confirm` through a pseudo terminal, as a person would."""
    master, slave = pty.openpty()
    proc = subprocess.Popen(base + ["level2", "confirm", str(copy)], stdin=slave, stdout=slave, stderr=slave, env=env, close_fds=True)
    os.close(slave)
    out, sent, end = b"", False, time.time() + CONFIRM_TIMEOUT
    while time.time() < end:
        ready, _, _ = select.select([master], [], [], 0.2)
        if ready:
            try:
                chunk = os.read(master, 4096)
            except OSError:
                break
            if not chunk:
                break
            out += chunk
        if not sent and time.time() > end - CONFIRM_TIMEOUT + CONFIRM_DELAY and out:
            os.write(master, copy.name.encode() + b"\n")
            sent = True
        if proc.poll() is not None and not ready:
            break
    proc.wait(timeout=10)
    os.close(master)
    if proc.returncode:
        sys.exit("level2 confirm failed:\n" + out.decode(errors="replace"))


def run(lang, project, src, tmp, opt):
    """Prepares the copy, registers the project, confirms level 2 and scans with the indexer. Returns (copy, data, report)."""
    copy = tmp / f"corpus-{lang}"
    shutil.copytree(src, copy)
    extra = prepare(lang, copy, tmp)
    env = environment(extra, Path(opt.tools_bin).resolve())
    data = tmp / "data"
    excluded = json.dumps(sorted({"bin", "obj", ".git", "node_modules", "target", "dist"} | set(project.get("skip_folders", []))))
    base = [opt.codeskimr, "--data-dir", str(data), "--languages", str(opt.languages), "--set", f"scan.excluded_folders={excluded}"]
    subprocess.run(base + ["scan", str(copy), "--json"], check=True, capture_output=True, env=env)
    confirm(base, copy, env)
    started = time.time()
    done = subprocess.run(base + ["--set", "level2.enabled=true"] + [a for kv in opt.set for a in ("--set", kv)] + ["scan", str(copy)], capture_output=True, text=True, env=env)
    seconds = round(time.time() - started, 1)
    report = [l for l in (done.stdout + done.stderr).splitlines() if "Level 2" in l or "RES-" in l or "error" in l.lower()]
    print(f"level 2 scan: exit {done.returncode}, {seconds} s: {report}", file=sys.stderr)
    if done.returncode or not any("checked" in l for l in report):
        sys.exit("level 2 did not run:\n" + done.stdout + done.stderr)
    return copy, data, {"seconds": seconds, "lines": report}
