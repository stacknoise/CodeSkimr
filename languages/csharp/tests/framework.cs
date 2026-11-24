namespace Demo;

public interface IStore { bool Exists(string name); }

public sealed class Store : IStore
{
    public bool Exists(string name) => File.Exists(name);
    public int IndexOf(string key) => 0;
}

public class Options { public string Directory { get; set; } public string File { get; set; } }

public class Finder
{
    private readonly List<string> _names = new();
    private readonly IStore _store;
    public Finder(IStore store) { _store = store; }

    public bool Framework(string path) { return !File.Exists(path); }
    public bool Directories(string path) { return Directory.Exists(path); }
    public int Listed(List<string> names) { return names.IndexOf("a"); }
    public int Field() { return _names.IndexOf("a"); }
    public int Local() { List<string> own = new(); return own.IndexOf("a"); }
    public bool Through(IStore s) { return s.Exists("a") || _store.Exists("b"); }
    public bool Own(Store s) { return s.Exists("c"); }
    public bool Unknown() { var x = Make(); return x.Exists("d"); }
    private static object Make() => null;
}

namespace Elsewhere
{
    public class Document { public string Name { get; set; } }
}

public partial class Parts
{
    public bool Reach() { return Config.Exists("a") && Settings.Exists("b"); }
}

public partial class Parts
{
    public Store Config { get; set; }
    private readonly Store Settings = new();
}

public class Strangers
{
    public Options File { get; set; }
    public bool Property(string path) { return File.Exists(path); }
}
