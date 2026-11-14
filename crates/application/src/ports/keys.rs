//! Reading the configuration and translation keys of a project (adapter: `codeskimr_source`).

use codeskimr_domain::{FolderPath, RelativePath};

use crate::config::KeySources;
use crate::error::CodedError;

/// What a key belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// A key of a TOML configuration file, as the dotted path from the top (`section.key`).
    Config,
    /// The id of a Fluent message.
    I18n,
}

/// One key and where it is declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEntry {
    /// The dotted key path or the message id.
    pub name: String,
    /// Configuration or translation.
    pub kind: KeyKind,
    /// File that declares it.
    pub path: RelativePath,
    /// Line of the declaration, zero-based.
    pub line: u32,
}

/// Lists the keys of the files below the folders of `sources`. Paths come from the folder walk,
/// never from the caller, and nothing outside the project root is read (blueprint S-05).
pub trait ProjectKeys: Send + Sync {
    /// Every key found, files in path order, keys in file order.
    ///
    /// A folder or a file that is missing, too large or unreadable is skipped.
    ///
    /// # Errors
    /// A coded failure of the `SRC` group if the root cannot be used at all.
    fn keys(&self, root: &FolderPath, sources: &KeySources) -> Result<Vec<KeyEntry>, CodedError>;
}
