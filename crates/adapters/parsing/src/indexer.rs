//! The `[indexer]` table of a language pack: how level 2 runs the indexer of the language
//! (ADR 0033). Every command line, version and install hint is language knowledge and lives in
//! the pack, never in code.

use codeskimr_application::ports::ToolVersion;
use codeskimr_application::ports::{
    IndexerSpec, PLACEHOLDER_OUTPUT, PLACEHOLDER_PROJECT, ToolSpec,
};
use std::collections::BTreeMap;

use serde::Deserialize;

use crate::consts::{
    INDEXER_BAD_PROGRAM, INDEXER_BAD_VERSION, INDEXER_NO_OUTPUT, INDEXER_UNKNOWN_PLACEHOLDER,
    NOTHING, PATH_SEPARATORS, PLACEHOLDER_CLOSE, PLACEHOLDER_OPEN,
};

/// A program the user installs, in `language.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifest {
    /// Executable name, without a folder.
    pub program: String,
    /// Arguments that print the version.
    pub version_args: Vec<String>,
    /// Oldest version that works, such as `0.2.0`.
    pub min_version: String,
}

/// `[indexer]` of `language.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexerManifest {
    /// Name of the indexer for messages.
    pub name: String,
    /// How the user installs the indexer.
    pub install_hint: String,
    /// The indexer program.
    pub tool: ToolManifest,
    /// What else has to be installed.
    #[serde(default)]
    pub toolchain: Vec<ToolManifest>,
    /// Arguments of the indexer; `{project}` and `{output}` are replaced.
    pub args: Vec<String>,
    /// Environment variables the indexer needs besides the base list.
    #[serde(default)]
    pub env_allow: Vec<String>,
    /// Environment variables set to a fixed value.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl ToolManifest {
    fn spec(&self) -> Result<ToolSpec, String> {
        if self.program.is_empty() || self.program.contains(PATH_SEPARATORS) {
            return Err([INDEXER_BAD_PROGRAM, self.program.as_str()].concat());
        }
        let min_version = ToolVersion::parse(&self.min_version)
            .ok_or_else(|| [INDEXER_BAD_VERSION, self.min_version.as_str()].concat())?;
        Ok(ToolSpec {
            program: self.program.clone(),
            version_args: self.version_args.clone(),
            min_version,
        })
    }
}

impl IndexerManifest {
    /// Checks the table and turns it into what the runner needs.
    ///
    /// # Errors
    /// The reason, if a program is a path, a version cannot be read, an argument holds a
    /// placeholder that does not exist, or no argument tells the indexer where to write.
    pub fn spec(&self) -> Result<IndexerSpec, String> {
        let mut writes_output = false;
        for arg in &self.args {
            writes_output |= arg.contains(PLACEHOLDER_OUTPUT);
            let rest = arg
                .replace(PLACEHOLDER_OUTPUT, NOTHING)
                .replace(PLACEHOLDER_PROJECT, NOTHING);
            if rest.contains(PLACEHOLDER_OPEN) || rest.contains(PLACEHOLDER_CLOSE) {
                return Err([INDEXER_UNKNOWN_PLACEHOLDER, arg.as_str()].concat());
            }
        }
        if !writes_output {
            return Err(INDEXER_NO_OUTPUT.to_owned());
        }
        Ok(IndexerSpec {
            name: self.name.clone(),
            indexer: self.tool.spec()?,
            toolchain: self
                .toolchain
                .iter()
                .map(ToolManifest::spec)
                .collect::<Result<_, _>>()?,
            args: self.args.clone(),
            env_allow: self.env_allow.clone(),
            env: self
                .env
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            install_hint: self.install_hint.clone(),
        })
    }
}
