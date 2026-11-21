import unittest

import check_expose_secret as check


class CountCalls(unittest.TestCase):
    def test_counts_product_calls_only(self):
        text = "a.expose_secret();\n// b.expose_secret()\nc.expose_secret(); d.expose_secret();\n"
        text += "#[cfg(test)]\nmod t { e.expose_secret(); }\n"
        self.assertEqual(check.count_calls(text), [1, 3, 3])

    def test_counts_path_calls_and_function_references(self):
        text = "let a = Credential::expose_secret(&x);\n"
        text += "let b = y.as_ref().map(Credential::expose_user);\n"
        text += "let c = <Credential>::expose_secret\n    (&z);\n"
        self.assertEqual(check.count_calls(text), [1, 2, 3])

    def test_definitions_and_other_names_are_no_calls(self):
        text = "pub fn expose_secret(&self) -> &str {\n"
        text += "    fn  expose_user() {}\n"
        text += "use secrecy::ExposeSecret;\nlet expose_secrets = 1; re_expose_user();\n"
        self.assertEqual(check.count_calls(text), [])

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
