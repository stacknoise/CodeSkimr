namespace Shop.Billing;

/// <summary>
///   Creates the invoices
///   of a shop.
/// </summary>
public class Invoicer
{
    /// <summary>Total of all invoices.</summary>
    /// <remarks>Not rounded.</remarks>
    public decimal Total { get; private set; }

    // Plain comments are no documentation.
    /// <inheritdoc/>
    public void Reset() { }

    [Obsolete("Use Reset")]
    protected abstract void Old();
}

enum Mode { A }
