//! The `[indexer]` table of a language pack: how level 2 runs the indexer of the language
//! (ADR 0033). Every command line, version and install hint is language knowledge and lives in
//! the pack, never in code.

use codeskimr_application::ports::ToolVersion;
use codeskimr_application::ports::{
    IndexerSpec, PLACEHOLDER_OUTPUT, PLACEHOLDER_PROJECT, ToolRequirement, ToolSpec,
};
use codeskimr_domain::{RequirementFormat, RequirementSpec};
use std::collections::BTreeMap;
use std::path::{Component, Path};

use serde::Deserialize;

use crate::consts::{
    INDEXER_BAD_HOME_DIR, INDEXER_BAD_PROGRAM, INDEXER_BAD_VERSION, INDEXER_NO_OUTPUT,
    INDEXER_UNKNOWN_PLACEHOLDER, NOTHING, PATH_SEPARATORS, PLACEHOLDER_CLOSE, PLACEHOLDER_OPEN,
};

/// The file in which a project names the version of a tool it needs, such as `global.json`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementManifest {
    /// The file, relative to the project root.
    pub file: String,
    /// `json`, `toml` or `text`.
    pub format: String,
    /// The keys down to the version in a `json` or `toml` file.
    #[serde(default)]
    pub path: Vec<String>,
}

impl RequirementManifest {
    fn spec(&self) -> Result<RequirementSpec, String> {
        let format = self
            .format
            .parse::<RequirementFormat>()
            .map_err(|_| self.format.clone())?;
        if self.file.trim().is_empty() {
            return Err(self.file.clone());
        }
        Ok(RequirementSpec {
            file: self.file.clone(),
            format,
            path: self.path.clone(),
        })
    }
}

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
    /// Where the project names the version it needs (the check of the tools reads it; the runner
    /// does not).
    #[serde(default)]
    pub requirement: Option<RequirementManifest>,
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
    /// Folders of the real home that the indexer needs, by variable: `CARGO_HOME = ".cargo"`.
    /// The indexer itself runs with an empty home.
    #[serde(default)]
    pub home_dirs: BTreeMap<String, String>,
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
    /// `home_dirs` as pairs; a folder must lie below the home, so no absolute path and no `..`.
    fn home_dirs(&self) -> Result<Vec<(String, String)>, String> {
        self.home_dirs
            .iter()
            .map(|(variable, folder)| {
                let path = Path::new(folder);
                let below_home = !folder.is_empty()
                    && path
                        .components()
                        .all(|part| matches!(part, Component::Normal(_)));
                if below_home {
                    Ok((variable.clone(), folder.clone()))
                } else {
                    Err([INDEXER_BAD_HOME_DIR, folder.as_str()].concat())
                }
            })
            .collect()
    }

    /// The files in which a project names the versions of the toolchain.
    ///
    /// # Errors
    /// The reason, if a file or format is empty or unknown.
    pub fn requirements(&self) -> Result<Vec<ToolRequirement>, String> {
        self.toolchain
            .iter()
            .filter_map(|tool| {
                let file = tool.requirement.as_ref()?;
                Some(file.spec().map(|file| ToolRequirement {
                    program: tool.program.clone(),
                    file,
                }))
            })
            .collect()
    }

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
            home_dirs: self.home_dirs()?,
            env: self
                .env
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            install_hint: self.install_hint.clone(),
        })
    }
}
