"""Unit tests for check_architecture.py (run: python3 -m unittest discover tools/ci)."""
import tempfile
import unittest
from pathlib import Path

import check_architecture as ca

RULES = {"allowed": {"a": ["b"], "b": []}, "wildcard": {"crates": ["app"]}}


class DirectionTests(unittest.TestCase):
    def test_allowed_dependency_passes(self):
        graph = {"a": {"b"}, "b": set(), "app": {"a", "b"}}
        self.assertEqual(ca.check_directions(RULES, graph), [])

    def test_forbidden_dependency_fails(self):
        graph = {"a": set(), "b": {"a"}, "app": set()}
        self.assertEqual(len(ca.check_directions(RULES, graph)), 1)

    def test_unlisted_crate_fails(self):
        self.assertEqual(len(ca.check_directions(RULES, {"c": set()})), 1)

    def test_stale_rule_fails(self):
        graph = {"a": set(), "app": set()}
        self.assertEqual(len(ca.check_rules_file(RULES, graph)), 2)


class StripTests(unittest.TestCase):
    def test_comments_and_strings_ignored(self):
        text = '// crate::x\n/* crate::y */ let s = "crate::z"; let r = r#"crate::w"#; use crate::ok;'
        out = ca.strip_rust(text)
        self.assertNotIn("crate::x", out)
        self.assertNotIn("crate::y", out)
        self.assertNotIn("crate::z", out)
        self.assertNotIn("crate::w", out)
        self.assertIn("crate::ok", out)

    def test_char_literal_quote(self):
        self.assertIn("crate::a", ca.strip_rust("let c = '\"'; use crate::a;"))


class EdgeTests(unittest.TestCase):
    TOPS = {"a", "b", "c"}

    def edges(self, text, path):
        return ca.module_edges(ca.strip_rust(text), path, self.TOPS)

    def test_simple_and_group(self):
        self.assertEqual(self.edges("use crate::b::X;", ["a"]), {"b"})
        self.assertEqual(self.edges("use crate::{b::X, c, self};", ["a"]), {"b", "c"})

    def test_super_chain(self):
        self.assertEqual(self.edges("use super::b::X;", ["a"]), {"b"})
        self.assertEqual(self.edges("use super::super::c::X;", ["a", "x"]), {"c"})
        self.assertEqual(self.edges("use super::y;", ["a", "x"]), {"a"})

    def test_self_path_is_own_module(self):
        self.assertEqual(self.edges("use self::y;", ["a", "x"]), {"a"})


class CycleTests(unittest.TestCase):
    def test_cycle_found(self):
        cycle = ca.find_cycle({"a": {"b"}, "b": {"c"}, "c": {"a"}})
        self.assertEqual(cycle, ["a", "b", "c", "a"])

    def test_acyclic(self):
        self.assertIsNone(ca.find_cycle({"a": {"b"}, "b": {"c"}, "c": set()}))

    def test_module_graph_from_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp)
            (src / "lib.rs").write_text("mod a; mod b;")
            (src / "a.rs").write_text("use crate::b::Y;")
            (src / "b").mkdir()
            (src / "b" / "mod.rs").write_text("use crate::a::X;")
            graph = ca.module_graph(src)
            self.assertEqual(graph, {"a": {"b"}, "b": {"a"}})
            self.assertIsNotNone(ca.find_cycle(graph))


if __name__ == "__main__":
    unittest.main()
