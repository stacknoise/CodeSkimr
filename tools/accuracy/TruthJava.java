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
 * Targets are tab-separated lines (file, line, name); the output
 * is one JSON array per target, each entry [file, line].
 */
public class TruthJava {
    /** Declaration key: file, line of the name, simple name. */
    record Key(String file, int line, String name) {}

    static Path root;
    static Trees trees;
    static final Map<Key, List<String>> uses = new HashMap<>();
    static final Map<Element, Key> keys = new HashMap<>();

    public static void main(String[] args) throws Exception {
        root = Path.of(args[0]).toAbsolutePath().normalize();
        List<String[]> targets = new ArrayList<>();
        try (BufferedReader in = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            for (String row; (row = in.readLine()) != null; ) targets.add(row.split("\t"));
        }
        List<Path> files = new ArrayList<>();
        try (var walk = Files.walk(root)) {
            walk.filter(p -> p.toString().endsWith(".java") && !p.getFileName().toString().equals("module-info.java")).sorted().forEach(files::add);
        }
        String classpath = args.length > 1 ? args[1] : "";
        if (args.length > 2 && !args[2].isEmpty() && Files.isDirectory(Path.of(args[2]))) {
            try (var walk = Files.walk(Path.of(args[2]))) {
                walk.filter(p -> p.toString().endsWith(".java")).sorted().forEach(files::add);
            }
        }
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
        StringBuilder out = new StringBuilder("[");
        for (int i = 0; i < targets.size(); i++) {
            String[] t = targets.get(i);
            int line = Integer.parseInt(t[1]);
            Set<String> found = new TreeSet<>();
            // The declaration may start above the name (annotations, modifiers): accept the next lines.
            for (int d = 0; d < 8 && found.isEmpty(); d++) {
                for (int l : new int[] {line + d, line - d}) {
                    List<String> hit = uses.get(new Key(t[0], l, t[2]));
                    if (hit != null) found.addAll(hit);
                }
            }
            out.append(i > 0 ? "," : "").append("[");
            int n = 0;
            for (String u : found) {
                int cut = u.lastIndexOf(':');
                out.append(n++ > 0 ? "," : "").append("[\"").append(u.substring(0, cut).replace("\\", "\\\\")).append("\",").append(u.substring(cut + 1)).append("]");
            }
            out.append("]");
        }
        System.out.println(out.append("]"));
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
            Key key = new Key(f, (int) cu.getLineMap().getLineNumber(pos), name);
            keys.put(e, key);
            return key;
        }

        void use(Tree tree) {
            Key k = key(trees.getElement(getCurrentPath()));
            if (k != null && file != null) uses.computeIfAbsent(k, x -> new ArrayList<>()).add(file + ":" + line(tree));
        }

        @Override public Void visitImport(ImportTree tree, Void v) { return null; }
        @Override public Void visitIdentifier(IdentifierTree tree, Void v) { use(tree); return super.visitIdentifier(tree, v); }
        @Override public Void visitMemberSelect(MemberSelectTree tree, Void v) { use(tree); return super.visitMemberSelect(tree, v); }
        @Override public Void visitMemberReference(MemberReferenceTree tree, Void v) { use(tree); return super.visitMemberReference(tree, v); }
        @Override public Void visitNewClass(NewClassTree tree, Void v) { use(tree); return super.visitNewClass(tree, v); }
    }
}
