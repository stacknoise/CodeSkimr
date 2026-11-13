# CodeSkimr: overview

CodeSkimr reads a code repository once and builds a database from it: which namespaces, types and members exist, who uses them, how big and complex they are and how the parts of the program depend on each other. You browse that database in the browser, ask it from the command line, or let a code agent ask it through MCP.

Everything runs on your machine. The server only listens on `127.0.0.1`; your code is not uploaded anywhere.

## The pieces

| Piece | What it is for | Chapter |
|---|---|---|
| Projects and scan | register a folder or a Git repository and read it | [Projects and scan](projects-and-scan.md) |
| Explorer and search | browse layers, namespaces, types and members; find symbols fast | [Explorer](explorer.md) |
| Dependency map | see the layers of your program and which one uses which | [Dependency map](dependency-map.md) |
| Usages | who uses this member, and what breaks if you change it | [Usages](usages.md) |
| Language indexers | make usages exact by letting the language's own tool check them | [Language indexers](indexers.md) |
| MCP server | give code agents the same knowledge | [MCP setup](mcp.md) |
| Sources and credentials | scan private Git repositories without putting secrets anywhere | [Sources and credentials](sources-and-credentials.md) |
| Error codes | what a code such as `SRC-401` means and what to do | [Error codes](errors.md) |

## Supported languages

Two tiers exist.

- **Full tier:** C#, Java, Kotlin, Python, Rust, TypeScript, TSX and JavaScript. Types, members, signatures, documentation comments, visibility, metrics and references.
- **Basic tier:** C, C++, Go, PHP and Ruby. Types, namespaces and functions with position and size, lines of code, complexity by decision points, and calls by name. No documentation comments and no visibility.

The scan screen shows the tier of every language it found in a project.

## Language of the program

The UI and the messages come in German and English. The browser UI has a switch `EN | DE` in the footer and a language field in the settings. The command line follows `CODESKIMR_LANG`, then `LC_ALL`, `LC_MESSAGES` and `LANG`: a value that starts with `de` gives German, everything else English. On Windows, where `LANG` is usually not set, use `CODESKIMR_LANG=de`.

## Look of the UI

Three themes exist: Hell, Petrol (default) and Dunkel. Choose one in the settings; the card shows a live preview. On first start a dark operating system gets Dunkel.
