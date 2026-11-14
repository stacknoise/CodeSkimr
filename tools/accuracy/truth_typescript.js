// Reference answers for TypeScript from the TypeScript language service: reads targets as JSON on
// stdin ({file, line (1-based), column (0-based), name}) and prints per target the [file, line]
// pairs of all other references inside the project folder (argument 2).
const path = require("path");
const fs = require("fs");
const ts = require(process.env.TYPESCRIPT_MODULE || "typescript");

const root = path.resolve(process.argv[2]);
const files = [];
(function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (/\.(ts|tsx)$/.test(entry.name)) files.push(full);
  }
})(root);

const options = { allowJs: true, target: ts.ScriptTarget.ES2022, moduleResolution: ts.ModuleResolutionKind.Bundler, module: ts.ModuleKind.ESNext, skipLibCheck: true, noEmit: true, strict: false };
const service = ts.createLanguageService({
  getScriptFileNames: () => files,
  getScriptVersion: () => "1",
  getScriptSnapshot: (f) => (fs.existsSync(f) ? ts.ScriptSnapshot.fromString(fs.readFileSync(f, "utf8")) : undefined),
  getCurrentDirectory: () => root,
  getCompilationSettings: () => options,
  getDefaultLibFileName: (o) => ts.getDefaultLibFilePath(o),
  fileExists: ts.sys.fileExists, readFile: ts.sys.readFile, readDirectory: ts.sys.readDirectory,
  directoryExists: ts.sys.directoryExists, getDirectories: ts.sys.getDirectories,
});

const targets = JSON.parse(fs.readFileSync(0, "utf8"));
const out = targets.map((t) => {
  const file = path.join(root, t.file);
  const text = fs.readFileSync(file, "utf8");
  const lines = text.split("\n");
  let offset = 0;
  for (let i = 0; i < t.line - 1; i++) offset += lines[i].length + 1;
  // A function written `const name = (...) => ...` starts behind its name: the name is the
  // identifier in front of the `=` that ends right before the symbol's start.
  const before = lines[t.line - 1].slice(0, t.column);
  const escaped = t.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const named = new RegExp(`(?:^|[^\\w$])(${escaped})\\s*(?::[^=]*)?=\\s*$`).exec(before);
  const at = named ? named.index + named[0].indexOf(t.name) : lines[t.line - 1].indexOf(t.name, t.column);
  const position = offset + (at >= 0 ? at : lines[t.line - 1].indexOf(t.name));
  const uses = new Set();
  for (const group of service.findReferences(file, position) || []) {
    for (const ref of group.references) {
      if (ref.isDefinition || !ref.fileName.startsWith(root) || ref.fileName.includes("node_modules")) continue;
      const source = ts.createSourceFile(ref.fileName, fs.readFileSync(ref.fileName, "utf8"), ts.ScriptTarget.Latest);
      const line = source.getLineAndCharacterOfPosition(ref.textSpan.start).line + 1;
      uses.add(JSON.stringify([path.relative(root, ref.fileName).split(path.sep).join("/"), line]));
    }
  }
  return [...uses].map((u) => JSON.parse(u));
});
console.log(JSON.stringify(out));
