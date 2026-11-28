//! The structure of a snapshot for the explorer tree (adapter: `codeskimr_storage`).

use codeskimr_domain::{
    LanguageId, QualifiedName, SnapshotId, SymbolId, SymbolKind, SymbolName, Visibility,
};

use crate::error::CodedError;

/// A namespace, module or type of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineNode {
    /// The symbol.
    pub id: SymbolId,
    /// Its simple name.
    pub name: SymbolName,
    /// Its qualified name.
    pub qualified_name: QualifiedName,
    /// Its kind.
    pub kind: SymbolKind,
    /// Enclosing symbol, absent for top-level nodes.
    pub parent: Option<SymbolId>,
    /// Number of direct members.
    pub members: u32,
    /// Language of the file that declares it.
    pub language: LanguageId,
    /// Folder of the file that declares it, relative to the project folder, with `/`; empty for
    /// the project folder itself. The package of an abapGit object (ADR 0113).
    pub folder: String,
}

/// The languages whose namespaces are the folders of their files: the packages of an abapGit
/// repository (ADR 0113). `OutlineNode::folder` is filled for them only. A first step; a setting
/// of the language pack replaces it (addendum of the ADR).
pub const FOLDER_NAMESPACE_LANGUAGES: [&str; 1] = ["abap"];

/// One row of the member table of a symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberRow {
    /// The member.
    pub id: SymbolId,
    /// Its simple name.
    pub name: SymbolName,
    /// Its qualified name: with its kind it is the key that annotations name it by (ADR 0068).
    pub qualified_name: QualifiedName,
    /// Its kind.
    pub kind: SymbolKind,
    /// Visibility, if the language has one.
    pub visibility: Option<Visibility>,
    /// Signature as written, if any.
    pub signature: Option<String>,
    /// Cyclomatic complexity, if measured.
    pub complexity: Option<f64>,
    /// Number of places that use it.
    pub usages: u32,
    /// First line of its declaration, counted from 0.
    pub first_line: u32,
    /// Last line of its declaration, counted from 0.
    pub last_line: u32,
}

/// Reads the structure of a snapshot.
pub trait CodeOutline: Send + Sync {
    /// All namespaces, modules and types of a snapshot, ordered by qualified name.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn outline(&self, snapshot: SnapshotId) -> Result<Vec<OutlineNode>, CodedError>;

    /// The direct members of a symbol as table rows, in declaration order (by start line, then column).
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn member_rows(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Vec<MemberRow>, CodedError>;
}
