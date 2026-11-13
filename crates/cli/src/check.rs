//! The output of `codeskimr check`: text for people, JSON for machines.

use codeskimr_application::conventions::{CheckReport, Detail, Finding, Location};
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    CLI_CHECK_DUPLICATE, CLI_CHECK_FINDING, CLI_CHECK_NOT_CHECKED, CLI_CHECK_SUMMARY,
    CLI_CHECK_TOO_LONG, CLI_CHECK_UNCHECKED,
};
use codeskimr_i18n::{FluentArgs, FluentValue, I18nError, Messages};
use serde::Serialize;

use crate::consts::{CODE_LIST_SEPARATOR, LINE_BREAK, arg};

/// The report as lines: every finding with its numbers, the files that were skipped, the rules
/// that were not checked, and a summary.
///
/// # Errors
/// `I18N-201` if a message is missing.
pub fn check_text(
    messages: &Messages,
    language: &str,
    report: &CheckReport,
) -> Result<String, I18nError> {
    let mut lines = Vec::new();
    for finding in &report.findings {
        lines.push(finding_line(messages, language, finding)?);
        lines.push(detail_line(messages, language, finding)?);
    }
    for skipped in &report.unchecked {
        let mut args = FluentArgs::new();
        args.set(arg::PATH, skipped.path.to_string());
        args.set(arg::CODE, skipped.code.id());
        args.set(arg::MESSAGE, say(messages, language, skipped.code)?);
        lines.push(messages.format(language, CLI_CHECK_UNCHECKED, Some(&args))?);
    }
    let mut summary = FluentArgs::new();
    summary.set(arg::FILES, FluentValue::from(report.files_checked));
    summary.set(arg::FINDINGS, FluentValue::from(report.findings.len()));
    lines.push(messages.format(language, CLI_CHECK_SUMMARY, Some(&summary))?);
    let codes: Vec<&str> = report.not_checked.iter().map(|code| code.id()).collect();
    let mut open = FluentArgs::new();
    open.set(arg::CODES, codes.join(CODE_LIST_SEPARATOR));
    lines.push(messages.format(language, CLI_CHECK_NOT_CHECKED, Some(&open))?);
    Ok(lines.join(LINE_BREAK))
}

fn say(messages: &Messages, language: &str, code: ErrorCode) -> Result<String, I18nError> {
    messages.format(language, code.message_key(), None)
}

fn one_based(location: &Location) -> u32 {
    location.span.start().line.saturating_add(1)
}

fn finding_line(
    messages: &Messages,
    language: &str,
    finding: &Finding,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::PATH, finding.location.path.to_string());
    args.set(arg::LINE, FluentValue::from(one_based(&finding.location)));
    args.set(arg::CODE, finding.code.id());
    args.set(arg::MESSAGE, say(messages, language, finding.code)?);
    messages.format(language, CLI_CHECK_FINDING, Some(&args))
}

fn detail_line(
    messages: &Messages,
    language: &str,
    finding: &Finding,
) -> Result<String, I18nError> {
    let mut args = FluentArgs::new();
    args.set(arg::NAME, finding.location.symbol.to_string());
    match &finding.detail {
        Detail::TooLong { lines, limit } => {
            args.set(arg::LINES, FluentValue::from(*lines));
            args.set(arg::LIMIT, FluentValue::from(*limit));
            messages.format(language, CLI_CHECK_TOO_LONG, Some(&args))
        }
        Detail::Duplicate {
            of,
            similarity_percent,
        } => {
            args.set(arg::ORIGINAL, of.symbol.to_string());
            args.set(arg::PATH, of.path.to_string());
            args.set(arg::LINE, FluentValue::from(one_based(of)));
            args.set(arg::PERCENT, FluentValue::from(*similarity_percent));
            messages.format(language, CLI_CHECK_DUPLICATE, Some(&args))
        }
    }
}

#[derive(Serialize)]
struct JsonPlace {
    path: String,
    line: u32,
    symbol: String,
}

#[derive(Serialize)]
struct JsonFinding {
    code: &'static str,
    #[serde(flatten)]
    at: JsonPlace,
    #[serde(skip_serializing_if = "Option::is_none")]
    lines: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duplicate_of: Option<JsonPlace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    similarity_percent: Option<u32>,
}

#[derive(Serialize)]
struct JsonUnchecked {
    path: String,
    code: &'static str,
}

#[derive(Serialize)]
struct JsonReport {
    files_checked: u32,
    findings: Vec<JsonFinding>,
    unchecked: Vec<JsonUnchecked>,
    not_checked: Vec<&'static str>,
}

fn place(location: &Location) -> JsonPlace {
    JsonPlace {
        path: location.path.to_string(),
        line: one_based(location),
        symbol: location.symbol.to_string(),
    }
}

fn json_finding(finding: &Finding) -> JsonFinding {
    let mut json = JsonFinding {
        code: finding.code.id(),
        at: place(&finding.location),
        lines: None,
        limit: None,
        duplicate_of: None,
        similarity_percent: None,
    };
    match &finding.detail {
        Detail::TooLong { lines, limit } => {
            json.lines = Some(*lines);
            json.limit = Some(*limit);
        }
        Detail::Duplicate {
            of,
            similarity_percent,
        } => {
            json.duplicate_of = Some(place(of));
            json.similarity_percent = Some(*similarity_percent);
        }
    }
    json
}

/// The report as one line of JSON. Lines are one-based.
///
/// # Errors
/// The serializer's error; plain numbers and text never fail.
pub fn check_json(report: &CheckReport) -> Result<String, serde_json::Error> {
    serde_json::to_string(&JsonReport {
        files_checked: report.files_checked,
        findings: report.findings.iter().map(json_finding).collect(),
        unchecked: report
            .unchecked
            .iter()
            .map(|skipped| JsonUnchecked {
                path: skipped.path.to_string(),
                code: skipped.code.id(),
            })
            .collect(),
        not_checked: report.not_checked.iter().map(|code| code.id()).collect(),
    })
}
