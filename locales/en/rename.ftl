# Dialog "Check a rename" of the explorer (ADR 0110).

rename-action = Check a rename…
rename-title = Check a rename
rename-subtitle = CodeSkimr changes nothing. It shows what would be affected.
rename-new-name = New name
rename-check = Check
rename-checking = Checking…
rename-hint = Type the new name and press Enter.
rename-invalid-name = That is not a usable new name. Use letters, digits and underscores, and not the old name.
rename-level2 = Level 2: places checked by the indexer
rename-level1 =
    { $unchecked ->
        [one] Level 1: { $unchecked } place is preliminary
       *[other] Level 1: { $unchecked } places are preliminary
    }
rename-tile-places = places to change
rename-tile-sure = { $count } sure
rename-tile-files = files
rename-tile-layers =
    { $count ->
        [one] { $count } layer
       *[other] { $count } layers
    }
rename-tile-conflicts =
    { $count ->
        [one] name conflict
       *[other] name conflicts
    }
rename-tile-texts = text hits
rename-tile-check-yourself = check yourself
rename-conflict-label = Conflict
rename-conflict-same = { $name } already exists in the same place with the same signature. After the rename there would be two symbols with this name and signature.
rename-conflict-other = { $name } already exists in the same place with another signature. That may be an overload; it depends on the language.
rename-public-label = Public
rename-public-text = The symbol is public. CodeSkimr does not see callers outside this project.
rename-sites = Code places · { $total }
rename-texts = Text hits that CodeSkimr cannot assign · { $total }
rename-more = … { $count } more
rename-not-searched = The project folder was not searched for text (Git source, or no right to read source).
rename-skipped =
    { $count ->
        [one] { $count } file could not be read.
       *[other] { $count } files could not be read.
    }
rename-site-sure = sure
rename-site-probable = probable
rename-site-unchecked = preliminary
rename-site-declaration = declaration
rename-text-documentation = docs
rename-text-configuration = configuration
rename-text-string = string
rename-text-nameof = nameof, sure
rename-text-comment = comment
rename-text-code = code
rename-footer = CodeSkimr does not say whether the code compiles afterwards. The compiler checks that.
rename-copy = Copy list
rename-json = Copy as JSON
rename-copied = Copied
rename-copy-failed = Not copied
rename-close = Close
rename-failed = The check did not work. Try again.
rename-esc = Esc
rename-by-file = by file
