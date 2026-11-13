//! The `FileAnalyzer` adapter: parses one file and extracts symbols and occurrences.

use std::fs;
use std::ops::ControlFlow;
use std::path::Path;
use std::time::{Duration, Instant};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AnalyzeRequest, FileAnalyzer, FileFacts};
use codeskimr_domain::{SourceFile, lines_of_code};
use tree_sitter::{ParseOptions, ParseState, Parser, Point, Tree};

use crate::encoding::decode;
use crate::error::ParsingError;
use crate::lines::classify;
use crate::pack::LanguagePack;
use crate::packs::LanguagePacks;
use crate::references;
use crate::symbols::{self, SymbolContext};

/// Tree-sitter implementation of the analyze stage (ADR 0013). The syntax tree never leaves it.
pub struct TreeSitterAnalyzer {
    packs: LanguagePacks,
    parse_timeout: Duration,
}

impl TreeSitterAnalyzer {
    /// Creates an analyzer over the installed packs; a file that takes longer than `parse_timeout`
    /// to parse fails with `PRS-501`.
    #[must_use]
    pub const fn new(packs: LanguagePacks, parse_timeout: Duration) -> Self {
        Self {
            packs,
            parse_timeout,
        }
    }

    fn run(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, ParsingError> {
        let pack = self.packs.get(&request.file.language)?;
        let location = Path::new(request.root.as_str()).join(request.file.path.as_str());
        let source = decode(&fs::read(location).map_err(ParsingError::Unreadable)?)?;
        let tree = parse(pack, &source, self.parse_timeout)?;
        let root = tree.root_node();
        let file = request.ids.file();
        let ctx = SymbolContext {
            pack,
            source: &source,
            file,
            path: &request.file.path,
            ids: request.ids,
        };
        let table = symbols::extract(&ctx, root)?;
        let pending = references::extract(&ctx, root, &table)?;
        Ok(FileFacts {
            file: SourceFile {
                id: file,
                snapshot: request.snapshot,
                path: request.file.path.clone(),
                language: request.file.language.clone(),
                content_hash: request.file.content_hash.clone(),
                loc: lines_of_code(classify(root, &pack.comment_nodes)),
            },
            symbols: table.symbols,
            pending,
        })
    }
}

impl FileAnalyzer for TreeSitterAnalyzer {
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        self.run(request).map_err(CodedError::caused_by)
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
