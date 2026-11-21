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

    def test_only_the_item_after_cfg_test_is_skipped(self):
        text = "#[cfg(test)]\nuse foo::bar;\nfn real(c: &C) { c.expose_secret(); }\n"
        text += "#[cfg(test)]\n#[allow(dead_code)]\nfn helper(c: &C) {\n    if x { c.expose_secret(); }\n}\n"
        text += "fn later(c: &C) { c.expose_user(); }\n"
        text += "#[cfg(test)] mod tests;\nfn last(c: &C) { c.expose_user(); }\n"
        self.assertEqual(check.count_calls(text), [3, 9, 11])

    def test_mut_variants_count(self):
        text = "a.expose_secret_mut();\nb.map(Secret::expose_user_mut);\nc.expose_secret_mutable();\n"
        self.assertEqual(check.count_calls(text), [1, 2])

    def test_braces_in_literals_do_not_hide_product_code(self):
        for literal in ('"{"', "'{'", 'r#"{"#', 'b"{{"', '"\\"{"', "'\\''; let b = '{'"):
            text = "#[cfg(test)]\nmod t { fn a() { let s = " + literal + "; } }\n"
            text += "fn p(x: &C) { x.expose_secret(); }\n"
            self.assertEqual(check.count_calls(text), [3], literal)

    def test_lifetimes_are_no_character_literals(self):
        text = "fn p<'a, 'b>(x: &'a C, y: &'b str) -> &'a str { x.expose_secret() }\n"
        self.assertEqual(check.count_calls(text), [1])

    def test_comments_and_literals_are_no_code(self):
        text = "/* a /* nested */\n#[cfg(test)]\n x.expose_secret() */\n"
        text += "fn p(x: &C) { x.expose_secret(); } // y.expose_secret()\n"
        text += 'let s = "z.expose_secret()";\n'
        self.assertEqual(check.count_calls(text), [4])

    def test_an_open_test_item_comment_or_raw_string_fails(self):
        for text in ("#[cfg(test)]\nmod t {\n", "fn a() {}\n/* open\n", 'let s = r#"open";\n'):
            with self.assertRaises(check.Unfinished, msg=text):
                check.count_calls(text)

    def test_marked_arms_variants_and_fields_end_at_their_own_end(self):
        real = "fn real(c: &C) { c.expose_secret(); }\n"
        arm = "fn f(x: E) {\n    match x {\n        E::A => 1,\n        #[cfg(test)]\n        E::B => 2\n    }\n}\n"
        self.assertEqual(check.count_calls(arm + real), [8])
        braced_arm = arm.replace("E::B => 2", "E::B => { x.expose_secret(); 2 }")
        self.assertEqual(check.count_calls(braced_arm + real), [8])
        field = "mod m {\n    struct S {\n        #[cfg(test)]\n        fake: [u8; 2]\n    }\n}\n"
        self.assertEqual(check.count_calls(field + real), [7])
        variant = "enum E {\n    #[cfg(test)]\n    Fake,\n}\nconst F: fn(&C) -> &str = C::expose_secret;\n"
        self.assertEqual(check.count_calls(variant + real), [5, 6])

    def test_code_behind_the_item_on_the_same_line_counts(self):
        text = "#[cfg(test)] use a::b; fn real(c: &C) { c.expose_secret(); }\n"
        text += "#[cfg(test)]\nfn helper(a: [u8; 2], c: &C) { c.expose_secret(); } fn more(c: &C) { c.expose_user(); }\n"
        self.assertEqual(check.count_calls(text), [1, 3])

    def test_raw_c_strings_are_literals(self):
        text = 'let a = cr"\\";\nlet n = x.expose_secret();\nlet b = "";\n'
        self.assertEqual(check.count_calls(text), [2])
        text = 'let a = cr#"a"b"#; x.expose_secret(); let b = c"q";\n'
        self.assertEqual(check.count_calls(text), [1])

    def test_a_string_that_is_left_over_fails(self):
        for text in ('let a = "open;\nx.expose_secret();\n', 'let a = zr"\\"; x.expose_secret(); let b = "";\n'):
            with self.assertRaises(check.Unfinished, msg=text):
                check.count_calls(text)

    def test_a_marked_last_parameter_does_not_hide_the_body(self):
        generic = "fn two<\n    #[cfg(test)] T\n>(x: &C) -> &str { x.expose_secret() }\n"
        self.assertEqual(check.count_calls(generic), [3])
        closure = "fn g(c: &C) {\n    run(|a,\n    #[cfg(test)] b\n    | c.expose_secret())\n}\n"
        self.assertEqual(check.count_calls(closure), [4])
        braced = closure.replace("| c.expose_secret())", "| { c.expose_secret() });")
        self.assertEqual(check.count_calls(braced), [4])

    def test_a_brace_in_the_type_of_a_marked_parameter_changes_nothing(self):
        generic = "fn bound<\n    #[cfg(test)] T: Tr<[u8; { 1 }]>\n>(x: &C) -> &str { x.expose_secret() }\n"
        self.assertEqual(check.count_calls(generic), [3])
        constant = generic.replace("T: Tr<[u8; { 1 }]>", "const N: [u8; { 1 }]")
        self.assertEqual(check.count_calls(constant), [3])
        closure = "fn g(c: &C) {\n    run(|a,\n    #[cfg(test)] b: [u8; { 2 }]\n    | c.expose_secret())\n}\n"
        self.assertEqual(check.count_calls(closure), [4])

    def test_angle_brackets_of_the_item_itself_do_not_end_it(self):
        text = "#[cfg(test)]\nimpl From<A> for B<C> {\n    fn from(a: A) -> Vec<Option<u8>> { a.expose_secret() }\n}\n"
        text += "#[cfg(test)]\nfn helper<T: Into<String>>(c: &C) -> Vec<String> {\n    c.expose_user()\n}\n"
        self.assertEqual(check.count_calls(text), [])

    def test_arrows_do_not_end_a_test_item(self):
        text = "#[cfg(test)]\nfn helper(c: &C) -> &str {\n    match c { _ => c.expose_secret() }\n}\n"
        self.assertEqual(check.count_calls(text), [])

    def test_an_unclear_end_counts_too_much_rather_than_too_little(self):
        text = "#[cfg(test)]\nfn user(c: &C) -> &str\nwhere\n    C: Clone,\n{\n    c.expose_user()\n}\n"
        self.assertEqual(check.count_calls(text), [6])

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
