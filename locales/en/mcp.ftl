# Texts of the MCP server. Tool descriptions are shown to code agents and stay in English in every language.

mcp-server-instructions = CodeSkimr is the reference for the code you write. Search for existing symbols before adding new ones, and read the architecture before deciding where code belongs.
mcp-tool-search-symbols = Find existing symbols (types, functions, constants) by name. Use it before writing new code to check whether something similar already exists. Returns ranked hits with kind and file.
mcp-tool-get-symbol = Show one symbol: signature, documentation, members, metrics and the architecture layer it belongs to. Accepts a symbol id or a qualified name.
mcp-tool-get-architecture = Show the architecture of a project: its layers with their namespaces and the dependencies between the layers, including violations and cycles.
mcp-tool-find-usages = Find where a symbol is used (direction incoming, the default) or what it uses (outgoing), up to a depth. Each result has the kind of use, the location and the confidence; a probable result is the resolver's best guess, so check it. References from tests are listed only with include_tests.
mcp-tool-impact-of-change = Show what a change of a symbol can reach: the affected symbols with their distance, layer and whether they are public interfaces. Use it before changing a signature or behaviour.
