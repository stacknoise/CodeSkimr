import unittest

import check_expose_secret as check


class CountCalls(unittest.TestCase):
    def test_counts_product_calls_only(self):
        text = "a.expose_secret();\n// b.expose_secret()\nc.expose_secret(); d.expose_secret();\n"
        text += "#[cfg(test)]\nmod t { e.expose_secret(); }\n"
        self.assertEqual(check.count_calls(text), [1, 3, 3])

    def test_test_files_are_skipped(self):
        from pathlib import Path
        self.assertTrue(check.is_test_file(Path("crates/x/tests/a.rs")))
        self.assertTrue(check.is_test_file(Path("crates/x/src/credentials_tests.rs")))
        self.assertFalse(check.is_test_file(Path("crates/x/src/credentials.rs")))


class Compare(unittest.TestCase):
    def test_new_changed_and_stale_entries_fail(self):
        found = {"a.rs": [1], "b.rs": [1, 2]}
        approved = {"b.rs": {"calls": 1}, "c.rs": {"calls": 1}}
        problems = check.compare(found, approved)
        self.assertEqual(len(problems), 3)

    def test_matching_list_passes(self):
        self.assertEqual(check.compare({"a.rs": [4]}, {"a.rs": {"calls": 1}}), [])


if __name__ == "__main__":
    unittest.main()
