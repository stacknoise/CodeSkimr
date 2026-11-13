// Fixture for the fingerprint tests: `total_price` and `sum_weights` have the same structure with
// other names and literals; `describe` is different; `tiny` is below the minimum length.

fn total_price(items: &[u32], rate: u32) -> u32 {
    let mut sum = 0;
    for item in items {
        if *item > 100 {
            sum += item * rate;
        } else {
            sum += item + 7;
        }
    }
    // The comment does not count.
    sum
}

fn sum_weights(weights: &[u32], factor: u32) -> u32 {
    let mut total = 5;
    for weight in weights {
        if *weight > 9 {
            total += weight * factor;
        } else {
            total += weight + 3;
        }
    }
    total
}

fn describe(name: &str) -> String {
    match name.len() {
        0 => String::from("empty"),
        1..=3 => format!("short {name}"),
        _ => name.to_uppercase(),
    }
}

fn tiny() -> u32 {
    1
}
