//! Consent and permission for running indexers (ADR 0033, decisions 9 and 10).

#[cfg(test)]
mod cases {
    use std::sync::Arc;

    use codeskimr_application::config::{Config, ConfigLayer, LayerSource, Level2};
    use codeskimr_application::error::CodedError;
    use codeskimr_application::level2::{
        ConsentRecord, ConsentStore, InMemoryConsent, authorize_run, confirm, revoke,
    };
    use codeskimr_application::ports::ToolVersion;
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{Permission, PermissionSet, Principal, ProjectId};

    const PROJECT: ProjectId = ProjectId::new(1);

    fn config(layers: &[(LayerSource, &str)]) -> Config {
        let layers: Vec<_> = layers
            .iter()
            .map(|(source, text)| ConfigLayer::parse(*source, text).unwrap())
            .collect();
        Config::resolve(&layers).unwrap()
    }

    fn code<T: std::fmt::Debug>(result: Result<T, CodedError>) -> ErrorCode {
        result.unwrap_err().code()
    }

    fn assert_not_confirmed(settings: &Level2, store: &Arc<dyn ConsentStore>) {
        assert_eq!(
            code(authorize_run(
                &Principal::local_owner(),
                settings,
                store,
                PROJECT
            )),
            ErrorCode::IndexerNotConfirmed
        );
    }

    #[test]
    fn by_default_nothing_runs() {
        let settings = config(&[]).level2;
        assert!(!settings.enabled);
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        assert_not_confirmed(&settings, &store);
    }

    #[test]
    fn enabling_in_the_user_config_alone_is_not_enough() {
        let settings = config(&[(LayerSource::User, "[level2]\nenabled = true\n")]).level2;
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        assert_not_confirmed(&settings, &store);
    }

    #[test]
    fn a_confirmation_alone_is_enough_because_it_enables_the_project() {
        let owner = Principal::local_owner();
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        confirm(&owner, store.as_ref(), PROJECT).unwrap();
        authorize_run(&owner, &config(&[]).level2, &store, PROJECT).unwrap();
        revoke(&owner, store.as_ref(), PROJECT).unwrap();
        assert_eq!(
            code(authorize_run(&owner, &config(&[]).level2, &store, PROJECT)),
            ErrorCode::IndexerNotConfirmed
        );
    }

    #[test]
    fn the_user_config_enables_for_every_project_but_each_needs_its_confirmation() {
        let owner = Principal::local_owner();
        let settings = config(&[(LayerSource::User, "[level2]\nenabled = true\n")]).level2;
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        store
            .store_consent(
                PROJECT,
                ConsentRecord {
                    enabled: false,
                    confirmed: true,
                },
            )
            .unwrap();
        authorize_run(&owner, &settings, &store, PROJECT).unwrap();
    }

    #[test]
    fn the_project_file_cannot_switch_level_2_on_or_raise_its_limits() {
        let project_file = "[level2]\nenabled = true\ntimeout_seconds = 99999\nmax_memory_mb = 999999\nnot_a_key = 1\n";
        let resolved = config(&[(LayerSource::Project, project_file)]);
        let defaults = config(&[]);
        assert_eq!(resolved.level2, defaults.level2);
        assert!(!resolved.level2.enabled);
        // Other keys of the same file still count.
        let mixed = config(&[(
            LayerSource::Project,
            "[level2]\nenabled = true\n[scan]\nparse_timeout_ms = 7\n",
        )]);
        assert_eq!(mixed.scan.parse_timeout_ms, 7);
        assert!(!mixed.level2.enabled);
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        assert_eq!(
            code(authorize_run(
                &Principal::local_owner(),
                &resolved.level2,
                &store,
                PROJECT
            )),
            ErrorCode::IndexerNotConfirmed
        );
    }

    #[test]
    fn the_user_config_may_set_the_limits() {
        let settings = config(&[(
            LayerSource::User,
            "[level2]\ntimeout_seconds = 30\nmax_memory_mb = 512\n",
        )])
        .level2;
        let limits = settings.limits();
        assert_eq!(limits.timeout.as_secs(), 30);
        assert_eq!(limits.max_memory_bytes, 512 * 1024 * 1024);
    }

    #[test]
    fn a_restricted_principal_may_neither_confirm_nor_start() {
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        let nobody = Principal::limited(PermissionSet::none());
        let reader = Principal::limited(PermissionSet::from_iter([
            Permission::Read,
            Permission::Scan,
            Permission::Manage,
            Permission::Admin,
        ]));
        for principal in [nobody, reader] {
            assert_eq!(
                code(confirm(&principal, store.as_ref(), PROJECT)),
                ErrorCode::IndexerNotPermitted
            );
            assert_eq!(
                code(revoke(&principal, store.as_ref(), PROJECT)),
                ErrorCode::IndexerNotPermitted
            );
            assert_eq!(
                code(authorize_run(
                    &principal,
                    &config(&[]).level2,
                    &store,
                    PROJECT
                )),
                ErrorCode::IndexerNotPermitted
            );
        }
        // Nothing was written by the refused calls.
        assert_eq!(store.consent(PROJECT).unwrap(), ConsentRecord::default());
    }

    #[test]
    fn starting_and_confirming_are_separate_rights() {
        let store: Arc<dyn ConsentStore> = Arc::new(InMemoryConsent::default());
        let starter = Principal::limited(PermissionSet::from_iter([Permission::RunIndexer]));
        let confirmer = Principal::limited(PermissionSet::from_iter([Permission::ConfirmIndexer]));
        confirm(&confirmer, store.as_ref(), PROJECT).unwrap();
        assert_eq!(
            code(authorize_run(
                &confirmer,
                &config(&[]).level2,
                &store,
                PROJECT
            )),
            ErrorCode::IndexerNotPermitted
        );
        authorize_run(&starter, &config(&[]).level2, &store, PROJECT).unwrap();
        assert_eq!(
            code(confirm(&starter, store.as_ref(), PROJECT)),
            ErrorCode::IndexerNotPermitted
        );
    }

    #[test]
    fn limits_are_validated() {
        for text in [
            "[level2]\ntimeout_seconds = 0\n",
            "[level2]\nmax_memory_mb = 0\n",
            "[level2]\nmax_output_bytes = 1\n",
        ] {
            let layer = ConfigLayer::parse(LayerSource::User, text).unwrap();
            assert!(Config::resolve(&[layer]).is_err(), "{text}");
        }
    }

    #[test]
    fn versions_are_read_from_the_output_of_tools() {
        let version = |text: &str| ToolVersion::find_in(text);
        assert_eq!(
            version("rust-analyzer 1.97.0 (abc 2026-06-30)"),
            ToolVersion::parse("1.97.0")
        );
        assert_eq!(version("v20.11.0"), ToolVersion::parse("20.11.0"));
        assert_eq!(
            version("openjdk version \"17.0.9\" 2023-10-17"),
            ToolVersion::parse("17.0.9")
        );
        assert_eq!(version("Python 3.12"), ToolVersion::parse("3.12.0"));
        assert!(version("2023-10-17 nothing").is_none());
        assert!(ToolVersion::parse("1.10.0") > ToolVersion::parse("1.9.9"));
        assert_eq!(ToolVersion::parse("1.8.0_392"), ToolVersion::parse("1.8.0"));
    }
}
