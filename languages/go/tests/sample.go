package shop

// Order is a purchase.
type Order struct {
	ID    int
	Total float64
}

// Pricer prices orders.
type Pricer interface {
	Price(o Order) float64
}

type Money = float64

type Code int

// NewOrder creates an order.
func NewOrder(id int) *Order {
	if id > 0 && id < 100 {
		return &Order{ID: id}
	}
	for i := 0; i < 3; i++ {
	}
	return nil
}

// Add adds to the total.
func (o *Order) Add(x float64) {
	println(NewOrder(1).Total)
	switch {
	case x > 0:
		o.Total += x
	default:
	}
}
