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

// The column of the identifier that names the symbol of `t`. Normally the symbol starts at its
// name. A declaration with modifiers (`export class`) or a function written
// `const name: Type = (...) => ...` starts in front of or behind its name; then the declaration
// that starts there (its initializer, if it has one) gives the name (the syntax tree,
// not text, so a function type in the annotation does no harm). Anything else fails loudly: a
// truth taken for the wrong identifier would count right answers as wrong.
function nameColumn(text, lines, t) {
  const line = lines[t.line - 1];
  if (line.startsWith(t.name, t.column)) return t.column;
  const source = ts.createSourceFile("x.ts", text, ts.ScriptTarget.Latest, true);
  const start = source.getPositionOfLineAndCharacter(t.line - 1, t.column);
  let found = null;
  const visit = (node) => {
    const named = node.name && node.name.getText(source) === t.name;
    const sameLine = named && source.getLineAndCharacterOfPosition(node.name.getStart(source)).line === t.line - 1;
    const afterStart = sameLine && node.name.getStart(source) >= start && node.getStart(source) <= start;
    const initial = named && node.initializer && node.initializer.getStart(source) === start;
    if (afterStart || initial) found = node.name.getStart(source);
    if (found === null) ts.forEachChild(node, visit);
  };
  visit(source);
  if (found === null) {
    throw new Error(`truth: no name \`${t.name}\` for the symbol at ${t.file}:${t.line}:${t.column}`);
  }
  return found - source.getPositionOfLineAndCharacter(t.line - 1, 0);
}

const targets = JSON.parse(fs.readFileSync(0, "utf8"));
const out = targets.map((t) => {
  const file = path.join(root, t.file);
  const text = fs.readFileSync(file, "utf8");
  const lines = text.split("\n");
  let offset = 0;
  for (let i = 0; i < t.line - 1; i++) offset += lines[i].length + 1;
  const position = offset + nameColumn(text, lines, t);
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
