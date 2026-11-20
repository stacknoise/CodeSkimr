import unittest

import check_case_clashes as check


class FindClashes(unittest.TestCase):
    def test_distinct_paths_pass(self):
        self.assertEqual(check.find_clashes(["a/b.rs", "a/c.rs", "d.rs"]), [])

    def test_same_path_listed_twice_passes(self):
        self.assertEqual(check.find_clashes(["a/b.rs", "a/b.rs"]), [])

    def test_files_differing_in_case_fail(self):
        self.assertEqual(check.find_clashes(["a/Preview.tsx", "a/preview.tsx"]),
                         [["a/Preview.tsx", "a/preview.tsx"]])

    def test_folders_differing_in_case_fail(self):
        clashes = check.find_clashes(["Foo/a.rs", "foo/b.rs"])
        self.assertEqual(clashes, [["Foo", "foo"]])

    def test_nested_folder_clash_fails(self):
        clashes = check.find_clashes(["x/Foo/a.rs", "x/foo/b.rs"])
        self.assertEqual(clashes, [["x/Foo", "x/foo"]])

    def test_file_and_folder_of_same_name_fail(self):
        clashes = check.find_clashes(["Foo", "foo/x.rs"])
        self.assertEqual(clashes, [["Foo", "foo"]])


if __name__ == "__main__":
    unittest.main()
