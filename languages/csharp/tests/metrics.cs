class M
{
    int PlainMethod() { return 1; }

    int Branches(int a, int b)
    {
        if (a > 0 && b > 0) { return 1; }
        else if (a < 0 || b < 0) { return 2; }
        var c = a ?? b;
        c ??= 3;
        return a > b ? a : b;
    }

    void Loops(int[] xs)
    {
        for (var i = 0; i < 3; i++) { }
        foreach (var x in xs) { }
        while (xs.Length > 0) { break; }
        do { } while (false);
    }

    string Cases(int k)
    {
        switch (k)
        {
            case 1:
            case 2:
                return "a";
            default:
                return "b";
        }
    }

    string Arms(int k) => k switch { 1 => "one", 2 => "two", _ => "other" };

    void Catches()
    {
        try { Work(); }
        catch (System.IO.IOException e) when (e != null) { }
        catch (System.Exception) { }
        finally { }
    }

    int Outer()
    {
        Func<int, int> f = x => x > 0 ? x : 0;
        int Local(int y) { if (y > 0) { return y; } return 0; }
        return Local(1);
    }
}

abstract class Shape
{
    public abstract int NoBody();
    int Field = 1 + 2;
}
