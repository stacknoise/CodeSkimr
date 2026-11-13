namespace Shop.Orders;

public class Order : Entity
{
    private readonly List<Line> lines = new List<Line>();
    private Printer printer;
    public int Total { get; set; }

    public Order(Printer printer, int total)
    {
        this.printer = printer;
        this.Total = total;
        Total = total;
    }

    public override void Save()
    {
        base.Save();
        var line = new Line();
        line.Add(Total);
        lines.Add(line);
        printer.Print(line);
        Helper.Log(line);
        foreach (Line each in lines)
        {
            each.Add(Total);
        }
        Action<int> show = x => x.ToString();
    }
}
