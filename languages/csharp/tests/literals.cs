namespace Demo;

public class Literals
{
    public int Clean(int[] items)
    {
        var total = 0;
        foreach (var item in items) { total += 1; }
        return total + items[0];
    }

    public double Numbers() => 42 + 2.5 + 0 + 1;

    public string Strings(string name, int count)
    {
        var label = "x" + 'y';
        return $"{name}!{count + 2}" + label + @"raw";
    }

    public int Local()
    {
        const int Limit = 99;
        return Limit + 7;
    }

    [Obsolete("old")]
    public bool Flags(object value) => value == null || true || false;
}
