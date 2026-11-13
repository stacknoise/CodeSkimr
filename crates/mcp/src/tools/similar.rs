//! `find_similar_code`.

use codeskimr_application::access;
use codeskimr_application::ports::{SimilarHit, SimilarQuery};
use codeskimr_domain::{Fingerprints, LanguageId, Principal, SymbolId};
use serde::Serialize;
use serde_json::Value;

use super::{Scope, SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::SimilarArgs;
use crate::consts::{
    ARG_LANGUAGE, ARG_MIN_SIMILARITY, ARG_SNIPPET, ARG_SYMBOL, ARG_SYMBOL_OR_SNIPPET, LINE_OFFSET,
    PERCENT_MAX, READ_TOOL_PERMISSION,
};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct Match {
    id: u64,
    qualified_name: String,
    kind: &'static str,
    path: String,
    line: u32,
    end_line: u32,
    similarity_percent: u32,
}

fn found(hit: &SimilarHit) -> Match {
    Match {
        id: hit.symbol.get(),
        qualified_name: hit.qualified_name.as_str().to_owned(),
        kind: hit.kind.label(),
        path: hit.path.as_str().to_owned(),
        line: hit.span.start().line + LINE_OFFSET,
        end_line: hit.span.end().line + LINE_OFFSET,
        similarity_percent: hit.percent,
    }
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    min_similarity_percent: u32,
    matches: Vec<Match>,
    notice: Option<CodeReport>,
}

/// What the caller wants compared: a stored function or a piece of code.
enum Subject<'a> {
    Symbol(&'a str),
    Snippet { code: &'a str, language: &'a str },
}

fn invalid(detail: &str) -> McpError {
    McpError::ArgumentsInvalid(detail.to_owned())
}

fn subject(args: &SimilarArgs) -> Result<Subject<'_>, McpError> {
    match (args.symbol.as_deref(), args.snippet.as_deref()) {
        (Some(symbol), None) => Ok(Subject::Symbol(symbol)),
        (None, Some(code)) => {
            let language = args
                .language
                .as_deref()
                .ok_or_else(|| invalid(ARG_LANGUAGE))?;
            Ok(Subject::Snippet { code, language })
        }
        _ => Err(invalid(ARG_SYMBOL_OR_SNIPPET)),
    }
}

impl Tools {
    /// Finds the stored functions that resemble a function or a piece of code, most similar
    /// first: "does this already exist?".
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` if the arguments do not name exactly
    /// one subject, the snippet is too large or too short, or the symbol has no fingerprints,
    /// `PRS-201` if no language pack is installed for the snippet, `MCP-201`, `MCP-202` and
    /// `MCP-203` for the project and the symbol, or the code of a failing port.
    pub fn find_similar_code(
        &self,
        principal: &Principal,
        args: &SimilarArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let subject = subject(args)?;
        let minimum = self.apply_similarity(args.min_similarity_percent)?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let (query, own) = self.query_of(&scope, &subject)?;
        let wanted = usize::try_from(limit.value).unwrap_or(usize::MAX);
        let mut hits = self.similarity().similar(
            scope.snapshot.id,
            &SimilarQuery {
                fingerprints: &query,
                minimum_percent: minimum,
                exclude: own,
                limit: limit.value.saturating_add(1),
            },
        )?;
        let cut = limit.clamped || hits.len() > wanted;
        hits.truncate(wanted);
        to_json(&Answer {
            snapshot: scope.info,
            min_similarity_percent: minimum,
            matches: hits.iter().map(found).collect(),
            notice: cut.then(|| truncation_notice(limit.value, None)),
        })
    }

    fn apply_similarity(&self, requested: Option<u32>) -> Result<u32, McpError> {
        let value = requested.unwrap_or(self.limits().default_similarity_percent);
        if value == 0 || value > PERCENT_MAX {
            return Err(invalid(ARG_MIN_SIMILARITY));
        }
        Ok(value)
    }

    /// The fingerprints to look for and the symbol that is not its own match.
    fn query_of(
        &self,
        scope: &Scope,
        subject: &Subject<'_>,
    ) -> Result<(Fingerprints, Option<SymbolId>), McpError> {
        match subject {
            Subject::Symbol(text) => {
                let symbol = self.find_symbol(scope, text)?.symbol.id;
                let prints = self
                    .similarity()
                    .fingerprints_of(scope.snapshot.id, symbol)?
                    .ok_or_else(|| invalid(ARG_SYMBOL))?;
                Ok((prints, Some(symbol)))
            }
            Subject::Snippet { code, language } => self
                .snippet_prints(code, language)
                .map(|prints| (prints, None)),
        }
    }

    fn snippet_prints(&self, code: &str, language: &str) -> Result<Fingerprints, McpError> {
        let largest = usize::try_from(self.limits().max_snippet_bytes).unwrap_or(usize::MAX);
        if code.len() > largest {
            return Err(invalid(ARG_SNIPPET));
        }
        let language = LanguageId::new(language).map_err(|_| invalid(ARG_LANGUAGE))?;
        let prints = self.snippets().fingerprint(&language, code)?;
        if prints.is_empty() {
            return Err(invalid(ARG_SNIPPET));
        }
        Ok(prints)
    }
}
