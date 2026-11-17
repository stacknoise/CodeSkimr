//! Building blocks that give the model its closed vocabulary: ids, validated texts and labelled enums.

/// Implements `Display` by displaying a value derived from `self`.
macro_rules! display_as {
    ($name:ident, $value:ident => $view:expr) => {
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let $value = self;
                ::std::fmt::Display::fmt(&$view, f)
            }
        }
    };
}

/// Defines an id newtype over `u64`. Ids of different entities can never be mixed up.
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            /// Wraps a raw id value.
            #[must_use]
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            /// The raw id value.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        $crate::vocabulary::display_as!($name, value => value.0);
    };
}

/// Defines a text newtype that is never empty.
macro_rules! text_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Creates the value from text.
            ///
            /// # Errors
            ///
            /// Returns [`ModelError::Empty`] if the text is empty or only whitespace.
            pub fn new(value: impl Into<String>) -> Result<Self, $crate::ModelError> {
                let text = value.into();
                if text.trim().is_empty() {
                    Err($crate::ModelError::Empty)
                } else {
                    Ok(Self(text))
                }
            }

            /// The text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        $crate::vocabulary::display_as!($name, value => value.0.as_str());

        impl ::std::str::FromStr for $name {
            type Err = $crate::ModelError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::new(text)
            }
        }
    };
}

/// Defines a text newtype for user input: never empty and at most as long as a limit that the caller
/// passes in (the limits come from the configuration, so they are not a constant of the model).
macro_rules! bounded_text_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            /// Creates the value from user input.
            ///
            /// # Errors
            ///
            /// [`ModelError::EmptyText`] if the text is empty or only whitespace, and
            /// [`ModelError::TooLong`] if it has more than `max_chars` characters.
            pub fn new(value: impl Into<String>, max_chars: u32) -> Result<Self, $crate::ModelError> {
                let text = value.into();
                if text.trim().is_empty() {
                    return Err($crate::ModelError::EmptyText);
                }
                if u32::try_from(text.chars().count()).map_or(true, |count| count > max_chars) {
                    return Err($crate::ModelError::TooLong);
                }
                Ok(Self(text))
            }

            /// Takes a text that was stored earlier. A limit that was lowered since then does not
            /// make a stored text invalid, so only emptiness is checked.
            ///
            /// # Errors
            ///
            /// [`ModelError::EmptyText`] if the text is empty or only whitespace.
            pub fn restore(value: impl Into<String>) -> Result<Self, $crate::ModelError> {
                Self::new(value, u32::MAX)
            }

            /// The text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        $crate::vocabulary::display_as!($name, value => value.0.as_str());
    };
}

/// Defines an enumeration whose variants have a fixed wire label (API, MCP, storage).
macro_rules! labeled_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$variant_meta:meta])* $variant:ident => $label:expr ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name {
            $( $(#[$variant_meta])* $variant ),+
        }

        impl $name {
            /// Every variant, in declaration order.
            pub const ALL: &'static [Self] = &[ $( Self::$variant ),+ ];

            /// The label used on the wire and in storage.
            #[must_use]
            pub const fn label(self) -> &'static str {
                match self {
                    $( Self::$variant => $label ),+
                }
            }
        }

        $crate::vocabulary::display_as!($name, value => value.label());

        impl ::std::str::FromStr for $name {
            type Err = $crate::ModelError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::ALL
                    .iter()
                    .copied()
                    .find(|variant| variant.label() == text)
                    .ok_or($crate::ModelError::UnknownLabel)
            }
        }
    };
}

pub(crate) use {bounded_text_type, display_as, id_type, labeled_enum, text_type};
