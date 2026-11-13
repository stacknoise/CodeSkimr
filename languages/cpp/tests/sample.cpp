namespace shop {

// An order.
class Order {
public:
    Order(int id) : id_(id) {}
    int total() const { return id_ > 0 ? id_ : 0; }
    void add(int x);
private:
    int id_;
};

struct Item { int n; };

enum class Kind { A, B };

void Order::add(int x) {
    while (x > 0) { x--; }
}

int price(const Order& o) {
    try { return o.total(); } catch (...) { return 0; }
}

template <typename T>
T twice(T v) { return v + v; }

}
