namespace Demo;

public static class Names
{
    public const string Arg = "a";
    public const string Ret = "r";
    public const string Case = "c";
    public const string Attr = "t";
    public const string Lambda = "l";
    public const string Init = "i";
    public const string Cond = "q";
}

public interface IOne { void Close(); int Size { get; } }
public interface ITwo { void Close(); }

public class Svc
{
    public IOne Make() => null;
    public static T Load<T>() => default;
}

public class Item { public int Count { get; set; } }

public class Caller
{
    public void Take(string s) { }
    public void A() { Take(Names.Arg); }
    public string B() { return Names.Ret; }
    public int C(string s)
    {
        switch (s)
        {
            case Names.Case: return 1;
            default: return 0;
        }
    }
    [System.Obsolete(Names.Attr)]
    public void D() { }
    public int E(Item item) { return item.Count; }
    public int F(IOne one) { return one.Size; }
    public System.Func<string> G() => () => Names.Lambda;
    public void H() { var o = new { Value = Names.Init }; var l = new[] { Names.Cond }; }
}

public class Client
{
    void Go(Svc svc)
    {
        var x = svc.Make();
        x.Close();
        var y = Svc.Load<ITwo>();
        y.Close();
        var z = (IOne)svc.Make();
        z.Close();
    }
}
