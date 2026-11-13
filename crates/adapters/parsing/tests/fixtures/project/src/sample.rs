use std::fmt::Display;

/// A point in the plane.
pub struct Point {
    x: i32,
}

enum Shape {
    Dot,
}

// Counts things.
pub fn count(point: Point) -> i32 {
    let text = "multi
line";
    helper(point.x)
}

fn helper(value: i32) -> i32 {
    /* block
       comment */
    value
}

impl Point {
    pub(crate) fn origin() -> Point {
        Point { x: 0 }
    }
}
