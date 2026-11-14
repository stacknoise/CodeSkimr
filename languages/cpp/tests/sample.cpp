namespace shop {

// An order.
class Order {
public:
    Order(int id) : id_(id) {}
    int total() const { return id_ > 0 ? id_ : 0; }
    void add(int x);
    ~Order() {}
    Order* self() { return this; }
    bool operator==(const Order& o) const { return id_ == o.id_; }
    template <typename T> void tm(T v) {}
private:
    int id_;
};

struct Item { int n; };

enum class Kind { A, B };

void Order::add(int x) {
    while (x > 0) { x--; }
}

Order* Order::make() { return nullptr; }

template <typename T> struct Box { T get(); };

template <typename T> T Box<T>::get() { return T(); }

struct Other { void add(int x); };

void Other::add(int x) {}

int price(const Order& o) {
    try { return o.total(); } catch (...) { return 0; }
}

template <typename T>
T twice(T v) { return v + v; }

}
