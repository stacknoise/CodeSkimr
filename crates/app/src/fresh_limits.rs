//! The limits of an indexer, read again from the configuration files at every run, so that a
//! value the UI just saved counts for the next run of the running server (ADR 0103).

use codeskimr_application::config::Level2;
use codeskimr_application::level2::LimitSource;
use codeskimr_application::ports::IndexLimits;
use codeskimr_cli::GlobalOptions;

use crate::config_files;
use crate::locations::Locations;

/// Reads the layers again for every run. The project file never sets `level2` values, so no
/// project folder is needed.
pub struct FreshLimits {
    pub(crate) global: GlobalOptions,
    pub(crate) locations: Locations,
    /// What the program started with, for when the files cannot be read now.
    pub(crate) started_with: Level2,
}

impl LimitSource for FreshLimits {
    fn limits_of(&self, indexer: &str) -> IndexLimits {
        match config_files::load(&self.locations, &self.global, None) {
            Ok(config) => config.level2.limits_of(indexer),
            Err(_) => self.started_with.limits_of(indexer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::UI_SETTINGS_FILE;

    const MIB: u64 = 1024 * 1024;

    fn locations(data: &std::path::Path) -> Locations {
        Locations {
            data_dir: data.to_path_buf(),
            languages_dir: std::path::PathBuf::new(),
            user_config: None,
            bases: Vec::new(),
            program_dir: None,
        }
    }

    fn source(folder: &std::path::Path) -> FreshLimits {
        let global = GlobalOptions::default();
        let locations = locations(folder);
        let started_with = config_files::load(&locations, &global, None)
            .unwrap()
            .level2;
        FreshLimits {
            global,
            locations,
            started_with,
        }
    }

    #[test]
    fn a_value_saved_after_the_start_counts_for_the_next_run() {
        let folder = tempfile::tempdir().unwrap();
        let limits = source(folder.path());
        let before = limits.limits_of("scip-typescript").max_memory_bytes;
        std::fs::write(
            folder.path().join(UI_SETTINGS_FILE),
            r#"{"indexer_memory_mb": {"scip-typescript": 8192}}"#,
        )
        .unwrap();
        assert_eq!(
            limits.limits_of("scip-typescript").max_memory_bytes,
            8192 * MIB
        );
        assert_eq!(limits.limits_of("rust-analyzer").max_memory_bytes, before);
    }

    #[test]
    fn a_flag_still_wins_over_the_saved_value() {
        let folder = tempfile::tempdir().unwrap();
        std::fs::write(
            folder.path().join(UI_SETTINGS_FILE),
            r#"{"indexer_memory_mb": {"scip-typescript": 8192}}"#,
        )
        .unwrap();
        let global = GlobalOptions {
            overrides: vec![(
                "level2.indexer_memory_mb.scip-typescript".to_owned(),
                "1024".to_owned(),
            )],
            ..GlobalOptions::default()
        };
        let locations = locations(folder.path());
        let started_with = config_files::load(&locations, &global, None)
            .unwrap()
            .level2;
        let limits = FreshLimits {
            global,
            locations,
            started_with,
        };
        assert_eq!(
            limits.limits_of("scip-typescript").max_memory_bytes,
            1024 * MIB
        );
    }
}
