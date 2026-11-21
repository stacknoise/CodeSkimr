namespace Demo;

public record Settings(bool Enabled, string Name);
public record Inner(int Start, bool IsOpen);
public record Entry(Inner Child, bool IsActive);

public class Holder
{
    public Settings Settings { get; }
    public Entry Pick() => null;
}

public class Reader
{
    public bool Negated(Holder h) => !h.Settings.Enabled;
    public bool Chain(Entry entry) => entry.IsActive;
    public bool Deep(Entry a) => a.Child.IsOpen;
    public bool Twice(Holder x) { if (!x.Settings.Enabled) return false; return !x.Settings.Enabled; }
    public bool Both(Entry c) { if (c.IsActive && c.Child.IsOpen) return true; return false; }
    public bool Returned(Holder h) { return h.Settings.Enabled; }
    public bool Picked(Holder h) => h.Pick().IsActive;
    public bool Local(Holder h) { var c = h.Pick(); return !c.IsActive; }
}

public interface IStore { int Load(string key); }

public class Consumer
{
    private readonly IStore _store;
    public Consumer(IStore store) { _store = store; }
    public int Go() => _store.Load("a");
}

public class ConsumerTests
{
    private readonly IStore _stub = Substitute.For<IStore>();
    public void Arranged() { var store = Substitute.For<IStore>(); store.Load("a").Returns(1); }
    public void Field() { _stub.Load(Arg.Any<string>()).Returns(2); _stub.Load("x"); }
    public void Wrapped() { Assert.True(new Consumer(_stub).Go() == 2); }
}

public interface IDisk { bool Exists(string path); }
public class Index { public int IndexOf(string key) => 0; }

public class Foreign
{
    public bool Static(string p) => File.Exists(p) && Directory.Exists(p);
    public int Listed(List<string> names) => names.IndexOf("a");
    public int Own(Index index) => index.IndexOf("a");
    public bool Disk(IDisk disk) => disk.Exists("a");
    public bool Unknown(string p) { var d = Open(p); return d.Exists(p); }
}

public class Counter { public int Total { get; set; } public int Step; }

public class Counting
{
    public void Bump(Counter c) { c.Total++; ++c.Step; c.Step--; }
}
