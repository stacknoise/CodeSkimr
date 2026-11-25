# CodeSkimr

CodeSkimr reads a code repository once and turns it into navigable documentation: which namespaces, types and members exist, who uses them, how big and complex they are and how the parts of the program depend on each other. You browse the result in the browser, ask it from the command line or the REST API, or let a code agent ask it through MCP.

Everything runs on your machine. The server listens on `127.0.0.1` only; your code is not uploaded anywhere.

> **Status:** 1.0. See [CHANGELOG.md](CHANGELOG.md) and the release notes in `docs/release/`.

## What it does

- **Explorer and quick search** over layers, namespaces, types and members.
- **Dependency map** of the layers of your program.
- **Usages:** who uses a member and what breaks if you change it, with a mark for how sure each hit is (verified, unchecked, presumed). Optional language indexers (level 2) make usages exact; they are off until you allow them per project.
- **Metrics and conventions:** size, complexity and coupling per symbol; `codeskimr check` reports files that break the code rules (length, duplicates).
- **MCP server** for code agents, over standard input and output (`codeskimr mcp`) or HTTP at `/mcp` of the running app; **HTML export** of a project as one self-contained file (`codeskimr export`).
- **Private Git repositories:** logins live in the keystore of your system and are created in the UI or with `codeskimr credential`; no secret is stored anywhere else.
- Languages: C#, Java, Kotlin, Python, Rust, TypeScript, TSX and JavaScript (full tier); C, C++, Go, PHP and Ruby (basic tier).
- The UI and the messages come in German and English; the UI has three themes (Hell, Petrol, Dunkel). Theme and language are kept by the server.

## Install

Releases carry one single file per system (Windows x86_64, Linux x86_64, macOS Apple Silicon) with a SHA-256 sum and a GitHub provenance attestation; UI and language packs are inside. The files are not signed yet (signing is planned, date open); the manual chapter "Installation" explains the SmartScreen and Gatekeeper warnings. Otherwise build from source:

```
cd web && npm ci && npm run build && cd ..
cargo build --release --locked -p codeskimr
```

Requirements and the full steps: [installation](docs/manual/en/installation.md).

## Use

```
codeskimr                 # start the local server and open the browser
codeskimr scan <folder>   # scan a folder or a Git URL and print a summary
codeskimr check           # check files against the code rules
codeskimr mcp             # MCP over standard input and output
codeskimr --help
```

## Documentation

- User manual: [English](docs/manual/en/index.md), [Deutsch](docs/manual/de/index.md)
- [Error code catalogue](docs/error-codes.md) (generated from `errors/error-codes.toml`), explained in the manual chapter [Error codes](docs/manual/en/errors.md)
- [Accuracy of usages per language](docs/accuracy.md)
- Development: [BLUEPRINT.md](BLUEPRINT.md) is the binding specification, [CLAUDE.md](CLAUDE.md) holds the rules for contributors and agents, decisions are in [docs/adr/](docs/adr/), CI is described in [docs/ci.md](docs/ci.md), releases in [docs/release.md](docs/release.md).
- Security: [SECURITY.md](SECURITY.md)

## License

[MIT](LICENSE)
