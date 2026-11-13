//! Sample for the symbol query of the Rust pack.

use std::fmt;

/// A public module.
pub mod shapes {
    /// A point on the plane.
    #[derive(Debug, Clone)]
    pub struct Point {
        /// The horizontal coordinate.
        pub x: f64,
        pub(crate) y: f64,
        label: String,
    }

    pub struct Marker(u8, pub u8);

    pub union Bits {
        pub whole: u32,
        parts: [u8; 4],
    }

    /// The ways to draw.
    pub enum Style {
        /// No line at all.
        Plain,
        Dashed(u8),
        Custom { width: u8, color: String },
    }

    /**
     * Something that has an area.
     * The second line.
     */
    pub trait Shape: fmt::Debug {
        /// The unit of the area.
        type Unit;
        const SIDES: usize;

        /// The area of the shape.
        fn area(&self) -> f64;

        fn describe(&self) -> String {
            format!("{:?}", self)
        }

        fn unit() -> &'static str;
    }

    impl Point {
        /// Creates a point.
        #[must_use]
        pub fn new(x: f64, y: f64) -> Self {
            Self { x, y, label: String::new() }
        }

        pub(crate) fn shifted(&self, dx: f64) -> Self {
            Self::new(self.x + dx, self.y)
        }

        pub(super) fn parent_only(&self) {}

        pub(in crate::shapes) fn in_path(&self) {}

        fn private_helper(&mut self) {}

        pub const fn origin() -> Self {
            Self { x: 0.0, y: 0.0, label: String::new() }
        }
    }

    impl Shape for Point {
        type Unit = u8;

        fn area(&self) -> f64 {
            0.0
        }

        fn unit() -> &'static str {
            "none"
        }
    }

    pub(crate) type Pair = (Point, Point);
    type Hidden = Vec<Point>;
}

/// The default scale.
pub const SCALE: f64 = 1.0;
const LIMIT: usize = 10;
pub static mut COUNTER: u32 = 0;

/// Doubles a number.
///
/// Adds nothing else.
pub fn double(value: i32) -> i32 {
    fn twice(inner: i32) -> i32 {
        inner * 2
    }
    twice(value)
}

#[inline]
fn private_function() {}

/// Runs in a block.
pub async fn run() {}

pub unsafe fn raw() {}

macro_rules! ignored {
    () => {};
}

extern "C" {
    fn external();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles() {
        assert_eq!(double(2), 4);
    }
}
