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

// Methods of different types with the same name do not collide: each belongs to its receiver type.
func (p Pricer2) Run() {}

func (q *Queue) Run() {}

func (c Code) String() string { return "code" }

func (b *Box[T]) Get() T { var zero T; return zero }

func (o Order) Run() {}
