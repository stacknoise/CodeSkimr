namespace Shop;

/// <summary>A basket of the shop: the project that the check of the export page scans.</summary>
public class Basket
{
    public int Total(int a, int b) { return a + b; }
}

public class Till
{
    public int Pay(Basket basket) { return basket.Total(1, 2); }
}
