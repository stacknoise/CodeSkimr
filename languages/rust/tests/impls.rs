//! Sample for `impl` blocks: their members belong to the type the block names (ADR 0039).

use std::fmt;

// An impl before the type is declared still belongs to the type; the type comes later.
impl Early {
    fn first(&self) -> u8 {
        0
    }
}

pub struct Early;

pub enum Mode {
    Fast,
    Slow,
}

impl Mode {
    fn hidden(&self) -> bool {
        matches!(self, Mode::Fast)
    }

    pub fn shown(&self) -> bool {
        !self.hidden()
    }
}

pub struct Holder<T> {
    item: T,
}

impl<T> Holder<T> {
    pub fn get(&self) -> &T {
        &self.item
    }
}

impl<T: fmt::Debug> fmt::Debug for Holder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.item)
    }
}

// Two blocks of one type with methods of the same name do not collide with other types.
impl Early {
    fn second(&self) -> u8 {
        1
    }
}

mod inner {
    // A path in the head (`super::Early`) gives no container: the method stays with this module.
    impl super::Early {
        pub fn elsewhere(&self) -> u8 {
            2
        }
    }

    impl Local {
        pub fn near(&self) -> u8 {
            3
        }
    }

    pub struct Local;
}

// A head that is not a plain type has no container; its members stay with the module.
impl fmt::Display for &Early {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "early")
    }
}
