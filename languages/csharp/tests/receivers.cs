namespace Demo;

public interface IReader { void Open(); }
public interface IWriter { void Open(); }
public class FileReader : IReader { public void Open() { } }
public class FileWriter : IWriter { public void Open() { } }

public class Editor { public void Apply(int x) { } }
public class RowEvent { public int Index { get; set; } }
public class Row { public int Index { get; set; } }

public class Base { public virtual int Key { get; set; } }
public class Derived : Base
{
    public override int Key { get => base.Key; set { base.Key = value; } }
}

public class Shapes(IReader primary, Editor tool)
{
    private readonly IReader _reader;
    private readonly List<Editor> _editors = new();

    public void NullForgiving(IReader? r) { r!.Open(); }
    public void Cast(object o) { ((IReader)o).Open(); }
    public void As(object o) { (o as IReader)!.Open(); }
    public void ThisField() { this._reader.Open(); }
    public async void Awaited(IReader r) { await r?.Open(); }
    public void Pattern(object o) { if (o is IReader p) p.Open(); }
    public void SwitchCase(object o) { switch (o) { case IReader s: s.Open(); break; } }
    public void OutVar() { Fill(out IReader got); got.Open(); }
    public void Primary() { primary.Open(); tool.Apply(1); }
    public void Unknown(object o) { Make().Open(); }

    public void Deconstruct() { var (a, b) = (new Editor(), 1); a.Apply(b); }
    public void DeconstructLoop() { foreach (var (k, v) in new Dictionary<int, Editor>()) v.Apply(k); }
    public void Query() { var r = from e in _editors let z = e select z.Apply(1); }
    public void Join() { var r = from e in _editors join f in _editors on e equals f into g select g.Apply(1); }

    public int Initializers() { var e = new RowEvent { Index = 3 }; return e.Index; }

    private void Fill(out IReader r) { r = new FileReader(); }
    private IReader Make() => new FileReader();
}
