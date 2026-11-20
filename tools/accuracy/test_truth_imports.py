"""Self-test of the reference scripts: a name in an import statement is no use, a real use on another line is.
The TypeScript script filters import declarations itself (the language service lists them); Jedi does not list them,
and the Python test keeps that assumption of the comparison checked.

Run with `python3 -m unittest discover -s tools/accuracy -p "test_*.py"`. Each test needs its reference tool
(Jedi; Node with the TypeScript module, `TYPESCRIPT_MODULE` as for `measure.py`) and is skipped without it.
"""
import json, os, shutil, subprocess, sys, tempfile, unittest
from pathlib import Path

HERE = Path(__file__).parent


def answer(command, root, targets):
    done = subprocess.run(command + [str(root)], input=json.dumps(targets), capture_output=True, text=True, check=True)
    return json.loads(done.stdout)


def has_jedi():
    return subprocess.run([sys.executable, "-c", "import jedi"], capture_output=True).returncode == 0


def has_typescript():
    node = shutil.which("node")
    code = "require(process.env.TYPESCRIPT_MODULE||'typescript')"
    return bool(node) and subprocess.run([node, "-e", code], capture_output=True).returncode == 0


class ImportsAreNoUses(unittest.TestCase):
    @unittest.skipUnless(has_jedi(), "Jedi is not installed")
    def test_python_reference_lists_no_import_line(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "a.py").write_text("def helper():\n    return 1\n")
            (root / "b.py").write_text("from a import (\n    helper,\n)\nimport a\n\nhelper()\n")
            target = {"file": "a.py", "line": 1, "column": 4, "name": "helper"}
            uses = answer([sys.executable, str(HERE / "truth_python.py")], root, [target])
            self.assertEqual(uses, [[["b.py", 6]]])

    @unittest.skipUnless(has_typescript(), "Node with TypeScript is not available")
    def test_typescript(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "a.ts").write_text("export function helper() {\n  return 1;\n}\n")
            (root / "b.ts").write_text('import {\n  helper,\n} from "./a";\n\nhelper();\n')
            target = {"file": "a.ts", "line": 1, "column": 16, "name": "helper"}
            uses = answer(["node", str(HERE / "truth_typescript.js")], root, [target])
            self.assertEqual(uses, [[["b.ts", 5]]])


if __name__ == "__main__":
    unittest.main()
