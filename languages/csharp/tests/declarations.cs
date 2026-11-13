using System;

namespace Shop.Orders
{
    /// <summary>An order of a customer.</summary>
    public sealed partial class Order : Entity, IOrder
    {
        public const int MaxLines = 100;
        private static readonly int First = 1, Second = 2;
        internal string note;

        public Order(int id) : base(id) { }

        public int Id { get; set; }

        protected internal string Title { get; init; }

        public event EventHandler Changed;

        public static async Task<int> CountAsync(string filter)
        {
            return 0;
        }

        public int this[int index] => index;

        public static Order operator +(Order left, Order right) => left;

        public override string ToString() => "order";

        private void Helper()
        {
            int Local(int value) => value;
        }

        private class Line { }
    }

    internal interface IOrder
    {
        void Submit();

        int Total { get; }
    }

    public struct Money { }

    public readonly record struct Point(int X, int Y);

    public record Customer(string Name);

    public enum Status
    {
        Open,
        Closed = 2,
    }

    public delegate void Handler(int value);

    class Hidden { }
}
