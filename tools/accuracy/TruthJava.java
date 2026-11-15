import com.sun.source.tree.*;
import com.sun.source.util.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.regex.*;
import javax.lang.model.element.*;
import javax.tools.*;

/**
 * Reference answers for Java from the compiler itself: javac parses and attributes the sources
 * (JavacTask.analyze), and every name that the compiler resolves to a declaration is a use of it.
 * It does not read the SCIP index of level 2 (ADR 0033, decision 14).
 *
 * Usage: java TruthJava.java source-root classpath extra-source-root < targets.tsv
 * `classpath` may be empty, `extra-source-root` holds generated sources that are compiled but never reported.
 * Targets are tab-separated lines (file, line, name, category); the category (`type`, `method`, `constructor`,
 * `field`) keeps a method and its parameter of the same name, or a class and its constructor, apart, because
 * the declarations are found by place and name and the window below would otherwise take the neighbour. The output
 * is one JSON array per target, each entry [file, line].
 */
public class TruthJava {
    /** Declaration key: file, line of the name, simple name. */
    record Key(String file, int line, String name, String category) {}

    static Path root;
    static Trees trees;
    static final Map<Key, List<String>> uses = new HashMap<>();
    static final Map<Element, Key> keys = new HashMap<>();

    public static void main(String[] args) throws Exception {
        root = Path.of(args[0]).toAbsolutePath().normalize();
        List<String[]> targets = readTargets();
        List<Path> files = sourceFiles(args);
        String classpath = args.length > 1 ? args[1] : "";
        analyze(files, classpath);
        StringBuilder out = new StringBuilder("[");
        for (int i = 0; i < targets.size(); i++) {
            out.append(i > 0 ? "," : "").append(answer(targets.get(i)));
        }
        System.out.println(out.append("]"));
    }

    /** The targets on standard input, one per row: file, line, name, category (tab separated). */
    static List<String[]> readTargets() throws Exception {
        List<String[]> targets = new ArrayList<>();
        try (BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            for (String row; (row = in.readLine()) != null; ) targets.add(row.split("\t"));
        }
        return targets;
    }

    /** The Java files below the root and, if given, below the extra source folder of the third argument. */
    static List<Path> sourceFiles(String[] args) throws Exception {
        List<Path> files = new ArrayList<>();
        try (var walk = Files.walk(root)) {
            walk.filter(p -> p.toString().endsWith(".java") && !p.getFileName().toString().equals("module-info.java")).sorted().forEach(files::add);
        }
        if (args.length > 2 && !args[2].isEmpty() && Files.isDirectory(Path.of(args[2]))) {
            try (var walk = Files.walk(Path.of(args[2]))) {
                walk.filter(p -> p.toString().endsWith(".java")).sorted().forEach(files::add);
            }
        }
        return files;
    }

    /** Parses and attributes the files, then records every declaration with its uses. */
    static void analyze(List<Path> files, String classpath) throws Exception {
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        StandardJavaFileManager fm = compiler.getStandardFileManager(null, null, StandardCharsets.UTF_8);
        JavacTask task = (JavacTask) compiler.getTask(null, fm, d -> { if (d.getKind() == Diagnostic.Kind.ERROR) System.err.println("javac: " + d.getMessage(null) + " @" + (d.getSource() == null ? "" : d.getSource().getName()) + ":" + d.getLineNumber()); }, List.of("-proc:none", "-Xlint:none", "--release", "17", "-classpath", classpath), null,
                fm.getJavaFileObjectsFromPaths(files));
        trees = Trees.instance(task);
        SourcePositions positions = trees.getSourcePositions();
        List<CompilationUnitTree> units = new ArrayList<>();
        for (CompilationUnitTree unit : task.parse()) units.add(unit);
        task.analyze();
        for (CompilationUnitTree unit : units) new Scanner(unit, positions).scan(unit, null);
    }

    /** The uses of one target as a JSON array of [path, line] pairs. */
    static String answer(String[] t) {
        int line = Integer.parseInt(t[1]);
        Set<String> found = new TreeSet<>();
        // The declaration may start above the name (annotations, modifiers): accept the next lines.
        // The nearest declaration of that name and category counts, whether or not it is used.
        boolean declared = false;
        for (int d = 0; d < 8 && !declared; d++) {
            for (int l : new int[] {line + d, line - d}) {
                List<String> hit = uses.get(new Key(t[0], l, t[2], t[3]));
                if (hit != null) {
                    found.addAll(hit);
                    declared = true;
                }
            }
        }
        StringBuilder out = new StringBuilder("[");
        int n = 0;
        for (String u : found) {
            int cut = u.lastIndexOf(':');
            out.append(n++ > 0 ? "," : "").append("[\"").append(u.substring(0, cut).replace("\\", "\\\\")).append("\",").append(u.substring(cut + 1)).append("]");
        }
        return out.append("]").toString();
    }

    /** Path of a compilation unit relative to the scanned root, or null for generated sources outside it. */
    static String relative(CompilationUnitTree unit) {
        Path path = Path.of(unit.getSourceFile().toUri()).toAbsolutePath().normalize();
        return path.startsWith(root) ? root.relativize(path).toString().replace('\\', '/') : null;
    }

    static class Scanner extends TreePathScanner<Void, Void> {
        final CompilationUnitTree unit;
        final SourcePositions positions;
        final String file;

        Scanner(CompilationUnitTree unit, SourcePositions positions) {
            this.unit = unit;
            this.positions = positions;
            this.file = relative(unit);
        }

        int line(Tree tree) {
            return (int) unit.getLineMap().getLineNumber(positions.getStartPosition(unit, tree));
        }

        int endLine(Tree tree) {
            return (int) unit.getLineMap().getLineNumber(Math.max(positions.getStartPosition(unit, tree), positions.getEndPosition(unit, tree) - 1));
        }

        /** Where the name of a declared element sits, found by reading the element's own tree. */
        Key key(Element e) {
            if (e == null) return null;
            Key known = keys.get(e);
            if (known != null) return known;
            TreePath path = trees.getPath(e);
            if (path == null) return null;
            Tree decl = path.getLeaf();
            CompilationUnitTree cu = path.getCompilationUnit();
            String f = relative(cu);
            if (f == null) return null;
            String name = e.getSimpleName().toString();
            if (e.getKind() == ElementKind.CONSTRUCTOR) name = e.getEnclosingElement().getSimpleName().toString();
            long pos = positions.getStartPosition(cu, decl);
            Key key = new Key(f, (int) cu.getLineMap().getLineNumber(pos), name, category(e.getKind()));
            keys.put(e, key);
            return key;
        }

        /** The category of a declaration, as the targets name it; parameters and locals have none of them. */
        static String category(ElementKind kind) {
            if (kind.isClass() || kind.isInterface()) return "type";
            if (kind == ElementKind.METHOD) return "method";
            if (kind == ElementKind.CONSTRUCTOR) return "constructor";
            if (kind == ElementKind.FIELD || kind == ElementKind.ENUM_CONSTANT) return "field";
            return "other";
        }

        void use(Tree tree) {
            use(tree, line(tree));
        }

        void use(Tree tree, int line) {
            Key k = key(trees.getElement(getCurrentPath()));
            if (k != null && file != null) uses.computeIfAbsent(k, x -> new ArrayList<>()).add(file + ":" + line);
        }

        /** The line of the last `length` characters of the tree: the name behind a dot, not the start of the receiver. */
        int nameLine(Tree tree, int length) {
            long end = positions.getEndPosition(unit, tree);
            if (end < 0) return line(tree);
            return (int) unit.getLineMap().getLineNumber(Math.max(positions.getStartPosition(unit, tree), end - length));
        }

        /** A declaration is known even if nothing uses it, so that a neighbour of the same name cannot stand in for it. */
        void declare(Tree tree) {
            Key k = key(trees.getElement(getCurrentPath()));
            if (k != null && file != null) uses.computeIfAbsent(k, x -> new ArrayList<>());
        }

        @Override public Void visitClass(ClassTree tree, Void v) { declare(tree); return super.visitClass(tree, v); }
        @Override public Void visitMethod(MethodTree tree, Void v) { declare(tree); return super.visitMethod(tree, v); }
        @Override public Void visitVariable(VariableTree tree, Void v) { declare(tree); return super.visitVariable(tree, v); }
        @Override public Void visitImport(ImportTree tree, Void v) { return null; }
        @Override public Void visitIdentifier(IdentifierTree tree, Void v) { use(tree); return super.visitIdentifier(tree, v); }
        @Override public Void visitMemberSelect(MemberSelectTree tree, Void v) { use(tree, nameLine(tree, tree.getIdentifier().length())); return super.visitMemberSelect(tree, v); }
        @Override public Void visitMemberReference(MemberReferenceTree tree, Void v) { use(tree, nameLine(tree, tree.getMode() == MemberReferenceTree.ReferenceMode.NEW ? 3 : tree.getName().length())); return super.visitMemberReference(tree, v); }
        @Override public Void visitNewClass(NewClassTree tree, Void v) { use(tree); return super.visitNewClass(tree, v); }
    }
}
