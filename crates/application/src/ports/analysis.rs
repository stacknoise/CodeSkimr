//! Understanding code (adapters: `codeskimr_parsing` and the metrics of `codeskimr_domain`).

use codeskimr_domain::{
    Confidence, FileId, Fingerprints, FolderPath, Metric, QualifiedName, Reference, ReferenceKind,
    SnapshotId, SourceFile, Span, Symbol, SymbolId,
};

use super::DiscoveredFile;
use crate::error::CodedError;
use crate::ids::IdSequence;

/// What to analyse in one file.
#[derive(Debug, Clone, Copy)]
pub struct AnalyzeRequest<'a> {
    /// Folder that holds the project files.
    pub root: &'a FolderPath,
    /// Snapshot the file belongs to.
    pub snapshot: SnapshotId,
    /// The file to analyse.
    pub file: &'a DiscoveredFile,
    /// Where ids for the file and its symbols come from.
    pub ids: &'a IdSequence,
}

/// What the syntax says about the thing in front of the dot of `receiver.name` (blueprint 9.2,
/// step 3). The adapter reports only what it can read from the syntax of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Receiver {
    /// The receiver is a variable whose declared type is written in the file, or a creation
    /// expression such as `new Foo()`.
    Typed(QualifiedName),
    /// The receiver is a name as written; it may be a type (static access) or a member.
    Named(QualifiedName),
    /// The receiver is the enclosing object itself (`this`, `self`).
    Own,
    /// The receiver is the base of the enclosing object (`base`, `super`).
    Base,
}

/// An occurrence of a name that is not yet mapped to a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingReference {
    /// Symbol that contains the occurrence.
    pub from: SymbolId,
    /// File of the occurrence.
    pub file: FileId,
    /// Range of the occurrence.
    pub span: Span,
    /// Kind of use.
    pub kind: ReferenceKind,
    /// The name as written, to be resolved. A name that equals the qualified name of a symbol
    /// stands for exactly that symbol (this is how references are resolved again).
    pub target: QualifiedName,
    /// What the syntax says about the receiver in front of the name, if there is one.
    pub receiver: Option<Receiver>,
    /// The confidence of an earlier resolution of this occurrence; the resolver never raises it.
    pub previous: Option<Confidence>,
}

/// The syntax fingerprints of one function-like symbol (blueprint 10.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolFingerprints {
    /// The function, method or constructor.
    pub symbol: SymbolId,
    /// Its winnowed fingerprints; never empty.
    pub fingerprints: Fingerprints,
}

/// Everything the analysis found in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFacts {
    /// The file as stored in the snapshot.
    pub file: SourceFile,
    /// Symbols declared in the file.
    pub symbols: Vec<Symbol>,
    /// Names used in the file that still need resolving.
    pub pending: Vec<PendingReference>,
    /// Fingerprints of the callable symbols that are long enough; empty if fingerprinting is off.
    pub fingerprints: Vec<SymbolFingerprints>,
}

/// Parses one file and extracts its symbols (Parse and Extract of blueprint section 7).
pub trait FileAnalyzer: Send + Sync {
    /// Analyses the file with the matching language pack.
    ///
    /// # Errors
    /// A coded failure such as `PRS-101`, `PRS-201` or `PRS-501`.
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError>;
}

/// Maps pending references to symbols (blueprint section 9).
pub trait ReferenceResolver: Send + Sync {
    /// Resolves `pending` against the global symbol table `symbols`. Names that match nothing are
    /// dropped; ambiguous ones become `Confidence::Probable`.
    ///
    /// # Errors
    /// A coded failure of the `RES` group.
    fn resolve(
        &self,
        symbols: &[Symbol],
        pending: &[PendingReference],
        ids: &IdSequence,
    ) -> Result<Vec<Reference>, CodedError>;
}

/// Computes the metrics of blueprint section 6.1 (strategy: interchangeable by configuration).
pub trait MetricsCalculator: Send + Sync {
    /// Measures every symbol.
    ///
    /// # Errors
    /// A coded failure if a value cannot be computed.
    fn measure(
        &self,
        symbols: &[Symbol],
        references: &[Reference],
    ) -> Result<Vec<Metric>, CodedError>;
}
