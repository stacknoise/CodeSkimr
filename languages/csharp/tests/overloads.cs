namespace Shop;

public class Order
{
    // Overloads share kind and name; their parameters tell them apart.
    public int Total(int a) { return a; }

    public int Total(
        int a, // first
        Map<string, int> b)
    {
        return a;
    }

    public abstract void Open();
}
