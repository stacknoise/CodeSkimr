//! The `FileAnalyzer` adapter: parses one file and extracts symbols and occurrences.

use std::borrow::Cow;
use std::fs;
use std::ops::ControlFlow;
use std::path::Path;
use std::time::{Duration, Instant};

use codeskimr_application::config::Similarity;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AnalyzeRequest, FileAnalyzer, FileFacts, SnippetFingerprinter};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Fingerprints, LanguageId, RelativePath, SourceFile, lines_of_code};
use tree_sitter::{ParseOptions, ParseState, Parser, Point, Tree};

use crate::content_hash;
use crate::encoding::decode;
use crate::error::ParsingError;
use crate::fingerprints::{self, FingerprintSettings};
use crate::lines::classify;
use crate::metrics;
use crate::null_conditional;
use crate::pack::LanguagePack;
use crate::packs::LanguagePacks;
use crate::references;
use crate::symbols::{self, SymbolContext};

/// Tree-sitter implementation of the analyze stage (ADR 0013). The syntax tree never leaves it.
pub struct TreeSitterAnalyzer {
    packs: LanguagePacks,
    parse_timeout: Duration,
    fingerprinting: Option<FingerprintSettings>,
    signature_limit: usize,
}

impl TreeSitterAnalyzer {
    /// Creates an analyzer over the installed packs; a file that takes longer than `parse_timeout`
    /// to parse fails with `PRS-501`.
    #[must_use]
    pub const fn new(packs: LanguagePacks, parse_timeout: Duration) -> Self {
        Self {
            packs,
            parse_timeout,
            fingerprinting: None,
            signature_limit: usize::MAX,
        }
    }

    /// Limits the signature of a symbol to `chars` characters (`scan.max_signature_chars`); a
    /// longer one is cut and ends with a mark. Without it signatures keep their full length.
    #[must_use]
    pub fn with_signature_limit(mut self, chars: u32) -> Self {
        self.signature_limit = usize::try_from(chars).unwrap_or(usize::MAX);
        self
    }

    /// Turns on syntax fingerprints for callables, as configured in the `[similarity]` section.
    /// Without it `FileFacts::fingerprints` stays empty.
    ///
    /// # Errors
    /// A model error if the configured sizes are zero, which a resolved config never has.
    pub fn with_fingerprints(mut self, similarity: &Similarity) -> Result<Self, ParsingError> {
        self.fingerprinting = Some(FingerprintSettings {
            params: similarity.fingerprint_params()?,
            min_tokens: usize::try_from(similarity.min_tokens).unwrap_or(usize::MAX),
        });
        Ok(self)
    }

    fn run(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, ParsingError> {
        let pack = self.packs.get(&request.file.language)?;
        let location = Path::new(request.root.as_str()).join(request.file.path.as_str());
        let source = decode(&fs::read(location).map_err(ParsingError::FileUnreadable)?)?;
        let source = prepared(pack, &source);
        let tree = parse(pack, &source, self.parse_timeout)?;
        let root = tree.root_node();
        let file = request.ids.file();
        let ctx = SymbolContext {
            pack,
            source: &source,
            file,
            path: &request.file.path,
            ids: request.ids,
            signature_limit: self.signature_limit,
        };
        let table = symbols::extract(&ctx, root)?;
        let pending = references::extract(&ctx, root, &table)?;
        let lines = classify(root, &pack.comment_nodes);
        let counts = metrics::counts_of(pack.metrics.as_ref(), root, &source, &table, &lines);
        let fingerprints = self.fingerprinting.map_or_else(Vec::new, |settings| {
            fingerprints::of_callables(pack, root, &table, settings)
        });
        let hashes = content_hash::of_symbols(pack, root, &source, &table)?;
        Ok(FileFacts {
            file: SourceFile {
                id: file,
                snapshot: request.snapshot,
                path: request.file.path.clone(),
                language: request.file.language.clone(),
                content_hash: request.file.content_hash.clone(),
                loc: lines_of_code(lines),
            },
            symbols: table.symbols,
            pending,
            fingerprints,
            counts,
            hashes,
        })
    }
}

impl FileAnalyzer for TreeSitterAnalyzer {
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        self.run(request).map_err(CodedError::caused_by)
    }

    fn naming_rules(&self) -> String {
        self.packs.naming_rules()
    }

    fn pack_load_time(&self) -> Duration {
        self.packs.load_time()
    }

    fn is_test_file(&self, language: &LanguageId, path: &RelativePath) -> bool {
        self.packs
            .get(language)
            .is_ok_and(|pack| pack.is_test_file(path.as_str()))
    }
}

impl SnippetFingerprinter for TreeSitterAnalyzer {
    fn fingerprint(&self, language: &LanguageId, source: &str) -> Result<Fingerprints, CodedError> {
        let settings = self
            .fingerprinting
            .ok_or_else(|| CodedError::new(ErrorCode::UnexpectedInternalError))?;
        let pack = self.packs.get(language).map_err(CodedError::caused_by)?;
        let source = prepared(pack, source);
        let tree = parse(pack, &source, self.parse_timeout).map_err(CodedError::caused_by)?;
        Ok(fingerprints::of_source(pack, tree.root_node(), settings))
    }
}

/// The source as the grammar of the pack can parse it, in the same width (ADR 0094).
fn prepared<'s>(pack: &LanguagePack, source: &'s str) -> Cow<'s, str> {
    if pack.blank_null_conditional_assignment {
        null_conditional::prepare(source)
    } else {
        Cow::Borrowed(source)
    }
}

fn parse(pack: &LanguagePack, source: &str, timeout: Duration) -> Result<Tree, ParsingError> {
    let mut parser = Parser::new();
    parser
        .set_language(&pack.language)
        .map_err(|error| ParsingError::invalid(pack.id().as_str(), error.to_string()))?;
    let started = Instant::now();
    let mut stop_when_late = |_: &ParseState| {
        if started.elapsed() > timeout {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let mut input = |offset: usize, _: Point| source.as_bytes().get(offset..).unwrap_or_default();
    let options = ParseOptions::new().progress_callback(&mut stop_when_late);
    parser
        .parse_with_options(&mut input, None, Some(options))
        .ok_or(ParsingError::Timeout)
}
