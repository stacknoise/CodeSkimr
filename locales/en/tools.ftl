# The tools that level 2 needs (ADR 0033). The language packs in languages/*/language.toml name
# these messages; they are shown for a tool that is missing, too old or of unknown version.

tool-scip-dotnet-purpose = scip-dotnet reads the exact usages of C# code from the compiler (level 2). Without it, usages are shown as unchecked.
tool-scip-dotnet-install = Install it with "dotnet tool install --global scip-dotnet".
tool-dotnet-purpose = The .NET SDK builds the C# project, which the indexer needs, in the version the project asks for (global.json).
tool-dotnet-install = Install the .NET SDK from dotnet.microsoft.com/download, in the version global.json names.

tool-scip-typescript-purpose = scip-typescript reads the exact usages of TypeScript and JavaScript code from the compiler (level 2). Without it, usages are shown as unchecked.
tool-scip-typescript-install = Install it with "npm install -g @sourcegraph/scip-typescript".
tool-node-purpose = Node.js runs the indexer and the project's own tooling, in the version the project asks for (.nvmrc).
tool-node-install = Install Node.js from nodejs.org or with a version manager, in the version .nvmrc names.

tool-scip-python-purpose = scip-python reads the exact usages of Python code from the type checker (level 2). Without it, usages are shown as unchecked.
tool-scip-python-install = Install it with "npm install -g @sourcegraph/scip-python".

tool-rust-analyzer-purpose = rust-analyzer reads the exact usages of Rust code (level 2). Without it, usages are shown as unchecked.
tool-rust-analyzer-install = Install it with "rustup component add rust-analyzer".
tool-cargo-purpose = Cargo and the Rust toolchain load the dependencies of the project, in the version the project asks for (rust-toolchain.toml).
tool-cargo-install = Install the Rust toolchain with rustup (rustup.rs), in the version rust-toolchain.toml names.

tool-scip-java-purpose = scip-java reads the exact usages of Java and Kotlin code from the compiler (level 2). Without it, usages are shown as unchecked.
tool-scip-java-install = See the installation guide of scip-java at github.com/sourcegraph/scip-java.
tool-java-purpose = A JDK builds the Java or Kotlin project, which the indexer needs.
tool-java-install = Install a JDK, for example from adoptium.net, and make sure "java" is on the search path.
