// Reference answers for C# from the compiler itself: Roslyn opens the restored solution, attributes every
// simple name and every `new`, and a name that resolves to a declaration is a use of it. It does not read a
// SCIP index (ADR 0033, decision 14), but scip-dotnet is built on Roslyn too: same engine (docs/accuracy.md).
//
// Usage: dotnet run --project tools/accuracy/truth_csharp -- WORKSPACE SOURCE < targets.json
// WORKSPACE holds the restored solution (any folder with one .sln), SOURCE is the scanned copy: only files
// that exist there are reported. Targets: [{file, line, name, category}]; the output is one array of
// [file, line] per target. A declaration is found by file, name and category within eight lines of the line
// of the target, because our symbols may start at attributes and modifiers.
using System.Text.Json;
using Microsoft.Build.Locator;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.MSBuild;

const int Window = 8;
MSBuildLocator.RegisterDefaults();
var workspaceRoot = Path.GetFullPath(args[0]).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
var sourceRoot = Path.GetFullPath(args[1]);
var targets = JsonSerializer.Deserialize<List<Dictionary<string, JsonElement>>>(Console.In.ReadToEnd())!;
var solutions = Directory.GetFiles(workspaceRoot, "*.sln");
if (solutions.Length != 1)
{
    Console.Error.WriteLine($"expected exactly one .sln in {workspaceRoot}, found {solutions.Length}");
    return 1;
}
var solution = solutions[0];

using var workspace = MSBuildWorkspace.Create();
workspace.WorkspaceFailed += (_, e) => { if (e.Diagnostic.Kind == WorkspaceDiagnosticKind.Failure) Console.Error.WriteLine("workspace: " + e.Diagnostic.Message); };
var opened = await workspace.OpenSolutionAsync(solution);

// declaration key -> reported uses
var uses = new Dictionary<(string File, int Line, string Name, string Category), SortedSet<(string, int)>>();

string? Relative(string path)
{
    var full = Path.GetFullPath(path);
    if (!full.StartsWith(workspaceRoot)) return null;
    var relative = Path.GetRelativePath(workspaceRoot, full).Replace('\\', '/');
    return File.Exists(Path.Combine(sourceRoot, relative)) ? relative : null;
}

string Category(ISymbol symbol) => symbol switch
{
    INamedTypeSymbol => "type",
    IMethodSymbol { MethodKind: MethodKind.Constructor } => "constructor",
    IMethodSymbol => "method",
    _ => "member",
};

void Use(ISymbol? symbol, SyntaxNode at, string file)
{
    if (symbol is null) return;
    symbol = symbol.OriginalDefinition;
    if (symbol is IMethodSymbol { ReducedFrom: { } reduced }) symbol = reduced;
    var declaration = symbol.DeclaringSyntaxReferences.FirstOrDefault();
    if (declaration is null) return;
    var declared = Relative(declaration.SyntaxTree.FilePath);
    if (declared is null) return;
    var key = KeyOf(symbol, declaration, declared);
    if (!uses.TryGetValue(key, out var set)) uses[key] = set = new();
    set.Add((file, at.SyntaxTree.GetLineSpan(at.Span).StartLinePosition.Line + 1));
}

// The name of an explicit interface implementation is the part behind the last dot.
(string, int, string, string) KeyOf(ISymbol symbol, SyntaxReference declaration, string declared)
{
    var name = symbol is IMethodSymbol { MethodKind: MethodKind.Constructor } ? symbol.ContainingType.Name : symbol.Name;
    name = name[(name.LastIndexOf('.') + 1)..];
    var line = declaration.SyntaxTree.GetLineSpan(declaration.Span).StartLinePosition.Line + 1;
    return (declared, line, name, Category(symbol));
}

// A declaration is known even if nothing uses it, so that a neighbour of the same name cannot stand in for it.
void Declare(ISymbol? symbol, string file)
{
    var declaration = symbol?.DeclaringSyntaxReferences.FirstOrDefault();
    if (symbol is null || declaration is null) return;
    var key = KeyOf(symbol, declaration, file);
    if (!uses.ContainsKey(key)) uses[key] = new();
}

foreach (var project in opened.Projects)
{
    foreach (var document in project.Documents)
    {
        var file = document.FilePath is null ? null : Relative(document.FilePath);
        if (file is null) continue;
        var model = await document.GetSemanticModelAsync();
        var root = await document.GetSyntaxRootAsync();
        if (model is null || root is null) continue;
        foreach (var node in root.DescendantNodes())
        {
            if (node is MemberDeclarationSyntax or LocalFunctionStatementSyntax or EnumMemberDeclarationSyntax)
                Declare(model.GetDeclaredSymbol(node), file);
            if (node is FieldDeclarationSyntax field)
                foreach (var variable in field.Declaration.Variables) Declare(model.GetDeclaredSymbol(variable), file);
            switch (node)
            {
                case IdentifierNameSyntax { IsVar: true }:
                    break;
                case SimpleNameSyntax name:
                    var info = model.GetSymbolInfo(name);
                    Use(info.Symbol, name, file);
                    break;
                case ObjectCreationExpressionSyntax or ImplicitObjectCreationExpressionSyntax:
                    Use(model.GetSymbolInfo(node).Symbol, node, file);
                    break;
            }
        }
    }
}

var result = new List<List<object[]>>();
foreach (var target in targets)
{
    var file = target["file"].GetString()!;
    var line = target["line"].GetInt32();
    var name = target["name"].GetString()!;
    var category = target["category"].GetString()!;
    var found = new SortedSet<(string, int)>();
    var declared = false;
    for (var d = 0; d < Window && !declared; d++)
        foreach (var l in new[] { line + d, line - d })
            if (uses.TryGetValue((file, l, name, category), out var hit))
            {
                found.UnionWith(hit);
                declared = true;
            }
    result.Add(found.Select(u => new object[] { u.Item1, u.Item2 }).ToList());
}
Console.WriteLine(JsonSerializer.Serialize(result));
return 0;
