// Fixture for the cognitive complexity and parameter counts (ADR 0048). The expected values are in
// crates/adapters/parsing/tests/rust_metrics.rs.

fn straight(a: i32) -> i32 {
    a + 1
}

fn nested(a: i32, b: i32) -> i32 {
    if a > 0 {
        for _ in 0..b {
            if a > b {
                return 1;
            }
        }
    }
    0
}

fn chain(a: i32) -> i32 {
    if a == 1 {
        1
    } else if a == 2 {
        2
    } else {
        3
    }
}

fn runs(a: bool, b: bool, c: bool, d: bool) -> bool {
    a && b && c || d
}

fn closure_nests(a: i32) -> i32 {
    let f = |x: i32| {
        if x > 0 {
            x
        } else {
            a
        }
    };
    f(a)
}

struct Shop;

impl Shop {
    fn method(&self, a: i32, b: i32, c: i32) -> i32 {
        match a {
            1 => b,
            _ => c,
        }
    }
}
