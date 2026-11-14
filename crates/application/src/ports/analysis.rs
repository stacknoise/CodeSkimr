//! Understanding code (adapters: `codeskimr_parsing` and the metrics of `codeskimr_domain`).

use codeskimr_domain::{
    Confidence, FileId, Fingerprints, FolderPath, LanguageId, Metric, QualifiedName, Reference,
    ReferenceKind, SnapshotId, SourceFile, Span, Symbol, SymbolCounts, SymbolId,
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
    /// There is a receiver, but the syntax does not tell its type: a variable without a written
    /// type (`var x = make()`), a call result or a chain. The member name alone may then belong
    /// to any type, so the resolver must not pick a scope member of the enclosing type for it.
    Unknown,
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
    /// The counts behind the metrics of the file's callables, types and module (ADR 0015).
    pub counts: Vec<SymbolCounts>,
}

/// Parses one file and extracts its symbols (Parse and Extract of blueprint section 7).
pub trait FileAnalyzer: Send + Sync {
    /// Analyses the file with the matching language pack.
    ///
    /// # Errors
    /// A coded failure such as `PRS-101`, `PRS-201` or `PRS-501`.
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError>;

    /// How the packs name symbols, as text that changes whenever a rule that changes the names
    /// does (ADR 0036). An incremental scan only keeps files of a snapshot with the same text.
    /// The default knows no rule.
    fn naming_rules(&self) -> String {
        String::new()
    }
}

/// Fingerprints a piece of code that is not part of a scanned file, for `find_similar_code`
/// (blueprint 10.4). It uses the same normalisation and parameters as the fingerprints of a scan,
/// so the values are comparable with the stored ones.
pub trait SnippetFingerprinter: Send + Sync {
    /// The fingerprints of `source`, which is parsed as the content of a file of `language`.
    /// The result is empty if the snippet is shorter than the configured minimum.
    ///
    /// # Errors
    /// `PRS-201` if no language pack is installed for `language`, `PRS-501` if parsing takes too
    /// long, or another coded failure of the `PRS` group.
    fn fingerprint(&self, language: &LanguageId, source: &str) -> Result<Fingerprints, CodedError>;
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
    /// Measures what the scan analysed. `symbols` and `references` are those of the whole
    /// snapshot, also of unchanged files in an incremental scan, so coupling is computed on
    /// complete data. `counts` come from the files that were analysed; the metrics of the other
    /// files stay as they are.
    ///
    /// # Errors
    /// A coded failure if a value cannot be computed.
    fn measure(
        &self,
        symbols: &[Symbol],
        references: &[Reference],
        counts: &[SymbolCounts],
    ) -> Result<Vec<Metric>, CodedError>;
}
