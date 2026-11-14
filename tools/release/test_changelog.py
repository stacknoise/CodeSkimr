"""Unit tests of changelog.py (run by CI: python3 -m unittest discover -s tools/release)."""
import unittest

import changelog


class ParseSubject(unittest.TestCase):
    def test_pr_number_is_split_off(self):
        self.assertEqual(changelog.parse_subject("feat: Scan abbrechen (#211)"), ("feat", "Scan abbrechen", "211", False))

    def test_scope_and_breaking_marker(self):
        kind, text, pr, breaking = changelog.parse_subject("fix(server)!: close the socket")
        self.assertEqual((kind, text, pr, breaking), ("fix", "close the socket", None, True))

    def test_rejects_german_title(self):
        with self.assertRaises(ValueError):
            changelog.parse_subject("fix: Oberfläche baut unter Windows (#285)")

    def test_rejects_free_text(self):
        with self.assertRaises(ValueError):
            changelog.parse_subject("Merge branch main")


class Render(unittest.TestCase):
    def test_groups_in_fixed_order_and_links(self):
        subjects = [("fix: b (#2)", "bbb"), ("feat: a (#1)", "aaa"), ("chore: c", "ccc")]
        text = changelog.render("1.0.0", "2027-01-01", subjects)
        self.assertLess(text.index("### Added"), text.index("### Fixed"))
        self.assertLess(text.index("### Fixed"), text.index("### Maintenance"))
        self.assertIn("- a ([#1](https://github.com/stacknoise/CodeSkimr/pull/1))", text)
        self.assertIn("- c (`ccc`)", text)
        self.assertTrue(text.startswith("## [1.0.0] - 2027-01-01"))

    def test_unknown_type_is_an_error(self):
        with self.assertRaises(ValueError):
            changelog.render("1.0.0", "2027-01-01", [("perf: x", "a")])

    def test_breaking_section_comes_first(self):
        text = changelog.render("1.0.0", "2027-01-01", [("feat!: new format (#3)", "a")])
        self.assertLess(text.index("### Breaking changes"), text.index("### Added"))


if __name__ == "__main__":
    unittest.main()
