//! Sample for the counting rules of `metrics.scm`: one case per rule of ADR 0015.

pub fn plain() -> i32 {
    1
}

// if, `&&`, `else if`, `||`; the plain `else` does not count
pub fn branches(a: i32, b: i32) -> i32 {
    if a > 1 && b > 1 {
        1
    } else if a < 0 || b < 0 {
        2
    } else {
        3
    }
}

// `if let` and `while let` count like `if` and `while`
pub fn patterns(items: Vec<Option<i32>>) -> i32 {
    let mut total = 0;
    if let Some(Some(first)) = items.first() {
        total += first;
    }
    let mut rest = items.into_iter();
    while let Some(Some(next)) = rest.next() {
        total += next;
    }
    total
}

// for and while count, `loop` does not
pub fn loops(limit: i32) -> i32 {
    let mut count = 0;
    for step in 0..limit {
        count += step;
    }
    while count > 100 {
        count -= 10;
    }
    loop {
        break;
    }
    count
}

// two arms count, the wildcard arm does not, also not with a guard
pub fn arms(value: i32) -> &'static str {
    match value {
        1 | 2 => "small",
        n if n > 10 => "big",
        _ => "other",
    }
}

// a guard of a wildcard arm adds the arm nothing, but its `&&` counts
pub fn wildcard_guard(a: i32, b: i32) -> i32 {
    match a {
        _ if a > 0 && b > 0 => 1,
        _ => 2,
    }
}

// the `?` operator, `let ... else`, `loop` and `unsafe` do not count
pub fn not_counted(text: &str) -> Option<i32> {
    let number = text.parse::<i32>().ok()?;
    let Some(double) = number.checked_mul(2) else {
        return None;
    };
    unsafe { Some(double) }
}

// the closure counts for its function
pub fn outer(values: &[i32]) -> Vec<i32> {
    fn local(value: i32) -> i32 {
        if value > 1 { value } else { 0 }
    }
    values.iter().map(|value| if *value > 3 { local(*value) } else { 0 }).collect()
}

pub fn calls(items: &[i32]) -> i32 {
    let first = items[0];
    first + items.len() as i32
}
