using System.Collections.Generic;
using Alias = Shop.Util;

namespace Shop.Orders;

public class Cart : Base<int>, IEnumerable<Item>
{
    private readonly List<Item> items = new List<Item>();
    private int count = Limits.Max;

    public Cart(int id) : base(id) { }

    public override void Run() { }

    public Item Find(Guid key, Item? fallback)
    {
        Helper.Log(key);
        count = items.Count + offset;
        count++;
        Total = Compute<int>(count);
        var name = typeof(Item).Name;
        var copy = fallback;
        return fallback ?? new Item();
    }
}
