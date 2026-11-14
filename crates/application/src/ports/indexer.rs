//! Running an external indexer for level 2 references (ADR 0033, decisions 1, 8 and 11; adapter:
//! `codeskimr_indexer`).
//!
//! Only the data lives here. What to run, which version it needs and how to install it is
//! language knowledge and comes from the language pack; the limits come from the configuration.

use std::any::Any;
use std::path::{Path, PathBuf};
use std::time::Duration;

use codeskimr_domain::LanguageId;

use crate::cancel::CancelToken;
use crate::error::CodedError;

use super::{RunPermit, ToolVersion};

pub use crate::consts::{PLACEHOLDER_OUTPUT, PLACEHOLDER_PROJECT};

/// One program the user has to install: the indexer itself or a tool of the language's toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSpec {
    /// Name of the executable, looked up in the `PATH` of the user (never in the project).
    pub program: String,
    /// Arguments that make it print its version.
    pub version_args: Vec<String>,
    /// Oldest version that works.
    pub min_version: ToolVersion,
}

/// How to run the indexer of one language, from `[indexer]` of the language pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexerSpec {
    /// Name of the indexer for messages, such as `scip-dotnet`.
    pub name: String,
    /// The indexer program.
    pub indexer: ToolSpec,
    /// What else has to be installed, such as the SDK or the runtime.
    pub toolchain: Vec<ToolSpec>,
    /// Arguments of the indexer. `{project}` and `{output}` are replaced; the indexer runs with the
    /// project root as its working folder and must write its index to `{output}`.
    pub args: Vec<String>,
    /// Names of environment variables the indexer needs besides the base list of the adapter.
    pub env_allow: Vec<String>,
    /// Variables set to a fixed value, such as the switch that stops `rustup` from installing a
    /// toolchain by itself.
    pub env: Vec<(String, String)>,
    /// How the user installs the indexer, shown when it is missing.
    pub install_hint: String,
}

/// Limits of one run, from `level2.*` of the configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexLimits {
    /// Longest run time; then the process group is killed.
    pub timeout: Duration,
    /// Most memory of the whole process tree, in bytes.
    pub max_memory_bytes: u64,
    /// Most bytes written in total: console output and files of the output folder.
    pub max_output_bytes: u64,
}

/// What to run for one language of one project.
#[derive(Debug, Clone, Copy)]
pub struct IndexJob<'a> {
    /// The language the index is for.
    pub language: &'a LanguageId,
    /// How to run its indexer.
    pub spec: &'a IndexerSpec,
    /// The folder of the project; the indexer runs in it.
    pub project_root: &'a Path,
    /// The limits of the run.
    pub limits: IndexLimits,
    /// Proof that consent and permission were checked; the runner reads the consent once more
    /// before it starts the process.
    pub permit: &'a RunPermit,
}

/// Why a tool is not usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolIssue {
    /// The program is not found.
    Missing,
    /// It is found but older than required, or its version cannot be read.
    TooOld {
        /// What it reports, shortened and redacted; empty if the output has no version.
        found: String,
        /// What the language pack requires.
        required: ToolVersion,
    },
}

/// One tool that is not usable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolProblem {
    /// Name of the program.
    pub program: String,
    /// What is wrong.
    pub issue: ToolIssue,
}

/// Whether an indexer can run on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexerStatus {
    /// The indexer and its toolchain are installed in a usable version.
    Ready,
    /// Something is missing or too old; level 1 stays.
    Unavailable(Vec<ToolProblem>),
}

/// The index file of a finished run. The folder it lives in is removed when this value is dropped.
pub struct IndexArtifact {
    path: PathBuf,
    // Held only for its `Drop`: removes the temporary output folder.
    _guard: Box<dyn Any + Send + Sync>,
}

impl IndexArtifact {
    /// An index at `path` that stays until `guard` is dropped.
    #[must_use]
    pub fn new(path: PathBuf, guard: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            path,
            _guard: guard,
        }
    }

    /// The index file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Debug for IndexArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(std::any::type_name::<Self>())
            .field(&self.path)
            .finish()
    }
}

/// Runs the indexer of a language. The job carries the [`RunPermit`] that only
/// [`crate::level2::authorize_run`] hands out.
pub trait IndexRunner: Send + Sync {
    /// Checks, without running any project code, whether the indexer and its toolchain exist and
    /// have a usable version.
    fn check(&self, spec: &IndexerSpec) -> IndexerStatus;

    /// Runs the indexer once and returns its index file.
    ///
    /// # Errors
    /// `SEC-301` if the consent was withdrawn since the permit was issued, `RES-201` if a tool is missing or too old, `RES-502` on timeout, `RES-503` when the output
    /// limit is exceeded, `RES-504` when the memory limit is exceeded, `RES-601` if it cannot be
    /// started or ends with an error, `PRJ-303` if `cancel` was raised. The process group is gone
    /// in every case.
    fn run(&self, job: &IndexJob<'_>, cancel: &CancelToken) -> Result<IndexArtifact, CodedError>;
}
