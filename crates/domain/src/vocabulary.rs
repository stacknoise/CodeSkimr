//! Building blocks that give the model its closed vocabulary: ids, validated texts and labelled enums.

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

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
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

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::ModelError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::new(text)
            }
        }
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

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.label())
            }
        }

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

pub(crate) use {id_type, labeled_enum, text_type};
