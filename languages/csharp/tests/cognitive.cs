// Fixture for the cognitive complexity and parameter counts (ADR 0048). The expected values are in
// crates/adapters/parsing/tests/csharp_metrics.rs.
using System;
using System.Linq;

class Cognitive
{
    int Straight(int a) { return a + 1; }

    int Nested(int a, int b)
    {
        if (a > 0)
        {
            for (var i = 0; i < b; i++)
            {
                if (a > b) { return 1; }
            }
        }
        return 0;
    }

    int Chain(int a)
    {
        if (a == 1) { return 1; }
        else if (a == 2) { return 2; }
        else { return 3; }
    }

    bool Runs(bool a, bool b, bool c, bool d)
    {
        return (a && b && c) || d;
    }

    int Handler(string x)
    {
        try { return int.Parse(x); }
        catch (FormatException)
        {
            if (x.Length == 0) { return 1; }
        }
        return 0;
    }

    int Cases(int a)
    {
        switch (a)
        {
            case 1: return 1;
            case 2: return 2;
            default: return 0;
        }
    }

    int Arms(int a)
    {
        return a switch { 1 => 10, 2 => 20, _ => 0 };
    }

    int Lambdas(int[] items)
    {
        return items.Select(x => x > 1 ? x : 0).Sum();
    }

    int Local(int a)
    {
        int Inner(int b)
        {
            if (b > 0) { return 1; }
            return 0;
        }
        return Inner(a);
    }

    int Loops(int a)
    {
        while (a > 0) { a--; }
        foreach (var x in new[] { 1, 2 })
        {
            foreach (var y in new[] { 3 })
            {
                if (x == y) { goto done; }
            }
        }
        done:
        return a;
    }

    Cognitive(int a, string b) { }

    int Defaults(int a, string b = "")
    {
        return a;
    }

    int Variadic(int a, params int[] rest)
    {
        return a + rest.Length;
    }
}
