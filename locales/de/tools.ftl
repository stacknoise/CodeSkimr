# Die Werkzeuge, die Stufe 2 braucht (ADR 0033). Die Sprachpakete in languages/*/language.toml
# nennen diese Meldungen; sie erscheinen bei einem Werkzeug, das fehlt, zu alt ist oder dessen
# Version nicht lesbar ist.

tool-scip-dotnet-purpose = scip-dotnet liest die genauen Verwendungen von C#-Code aus dem Compiler (Stufe 2). Ohne es werden Verwendungen als ungeprüft angezeigt.
tool-scip-dotnet-install = Installation mit "dotnet tool install --global scip-dotnet".
tool-dotnet-purpose = Das .NET SDK baut das C#-Projekt, was der Indexer braucht, in der Version, die das Projekt verlangt (global.json).
tool-dotnet-install = Das .NET SDK von dotnet.microsoft.com/download installieren, in der Version, die global.json nennt.

tool-scip-typescript-purpose = scip-typescript liest die genauen Verwendungen von TypeScript- und JavaScript-Code aus dem Compiler (Stufe 2). Ohne es werden Verwendungen als ungeprüft angezeigt.
tool-scip-typescript-install = Installation mit "npm install -g @sourcegraph/scip-typescript".
tool-node-purpose = Node.js führt den Indexer und die eigenen Werkzeuge des Projekts aus, in der Version, die das Projekt verlangt (.nvmrc).
tool-node-install = Node.js von nodejs.org oder mit einem Versionsmanager installieren, in der Version, die .nvmrc nennt.

tool-scip-python-purpose = scip-python liest die genauen Verwendungen von Python-Code aus dem Typprüfer (Stufe 2). Ohne es werden Verwendungen als ungeprüft angezeigt.
tool-scip-python-install = Installation mit "npm install -g @sourcegraph/scip-python".
tool-python3-purpose = Python führt die Analyse des Indexers in der Umgebung des Projekts aus, in der Version, die das Projekt verlangt (.python-version).
tool-python3-install = Python 3 von python.org oder mit einem Versionsmanager installieren, in der Version, die .python-version nennt.

tool-rust-analyzer-purpose = rust-analyzer liest die genauen Verwendungen von Rust-Code (Stufe 2). Ohne es werden Verwendungen als ungeprüft angezeigt.
tool-rust-analyzer-install = Installation mit "rustup component add rust-analyzer".
tool-cargo-purpose = Cargo und die Rust-Toolchain laden die Abhängigkeiten des Projekts, in der Version, die das Projekt verlangt (rust-toolchain.toml).
tool-cargo-install = Die Rust-Toolchain mit rustup (rustup.rs) installieren, in der Version, die rust-toolchain.toml nennt.

tool-scip-java-purpose = scip-java liest die genauen Verwendungen von Java- und Kotlin-Code aus dem Compiler (Stufe 2). Ohne es werden Verwendungen als ungeprüft angezeigt.
tool-scip-java-install = Siehe die Installationsanleitung von scip-java unter github.com/sourcegraph/scip-java.
tool-java-purpose = Ein JDK baut das Java- oder Kotlin-Projekt, was der Indexer braucht.
tool-java-install = Ein JDK installieren, zum Beispiel von adoptium.net, und dafür sorgen, dass "java" im Suchpfad liegt.
