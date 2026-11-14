//! `codeskimr level2 confirm`: who may confirm, what the user is shown, what they must type and
//! that nothing is stored if the project changed while they were asked (ADR 0033, decision 10).

#[cfg(test)]
mod cases {
    use std::cell::RefCell;
    use std::num::NonZeroU32;
    use std::path::Path;

    use codeskimr_application::level2::{self, ConsentRecord, ConsentStore};
    use codeskimr_cli::{GlobalOptions, Level2Action};
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{
        BranchName, FolderPath, GitUrl, Permission, Project, ProjectId, ProjectName, ProjectSource,
    };
    use codeskimr_storage::SqliteStore;
    use tempfile::TempDir;

    use crate::level2_consent::{Done, Target, run};
    use crate::locations::Locations;
    use crate::principal;

    const ID: u64 = 4;
    const NAME: &str = "shop";
    const FOLDER: &str = "/work/shop";
    const OTHER_FOLDER: &str = "/work/other";
    const REPOSITORY: &str = "https://example.org/shop.git";
    const BRANCH: &str = "release";

    struct Fixture {
        root: TempDir,
        locations: Locations,
    }

    fn project(source: ProjectSource) -> Project {
        Project {
            id: ProjectId::new(ID),
            name: ProjectName::new(NAME).unwrap(),
            source,
            layers: Vec::new(),
        }
    }

    fn folder(path: &str) -> ProjectSource {
        ProjectSource::Folder(FolderPath::new(path).unwrap())
    }

    fn git(branch: &str) -> ProjectSource {
        ProjectSource::Git {
            url: GitUrl::new(REPOSITORY).unwrap(),
            branch: BranchName::new(branch).unwrap(),
            credential: None,
        }
    }

    impl Fixture {
        fn with(source: ProjectSource) -> Self {
            let root = tempfile::tempdir().unwrap();
            let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages");
            let locations = Locations {
                bases: Vec::new(),
                user_config: None,
                languages_dir: languages,
                data_dir: root.path().join("data"),
            };
            let fixture = Self { root, locations };
            fixture.store().save_project(&project(source)).unwrap();
            fixture
        }

        fn store(&self) -> SqliteStore {
            SqliteStore::open(&self.locations.database(), NonZeroU32::MIN).unwrap()
        }

        fn consent(&self) -> ConsentRecord {
            self.store().consent(ProjectId::new(ID)).unwrap()
        }

        fn confirm(
            &self,
            argument: &str,
            ask: &dyn Fn(&Target) -> Result<String, codeskimr_application::error::CodedError>,
        ) -> Result<Done, codeskimr_application::error::CodedError> {
            let action = Level2Action::Confirm {
                project: argument.to_owned(),
            };
            run(&action, &GlobalOptions::default(), &self.locations, ask)
        }
    }

    #[test]
    fn a_client_of_mcp_or_the_server_may_neither_confirm_nor_run_an_indexer() {
        let fixture = Fixture::with(folder(FOLDER));
        let client = principal::remote_client();
        assert!(!client.holds(Permission::ConfirmIndexer));
        assert!(!client.holds(Permission::RunIndexer));
        assert!(client.holds(Permission::Scan));
        let store = fixture.store();
        let refused = level2::confirm(&client, &store, ProjectId::new(ID)).unwrap_err();
        assert_eq!(refused.code(), ErrorCode::IndexerNotPermitted);
        let refused = level2::revoke(&client, &store, ProjectId::new(ID)).unwrap_err();
        assert_eq!(refused.code(), ErrorCode::IndexerNotPermitted);
        assert_eq!(fixture.consent(), ConsentRecord::default());
        assert!(principal::resolve().holds(Permission::ConfirmIndexer));
    }

    #[test]
    fn the_user_is_shown_name_place_and_branch_and_confirms_by_typing_the_name() {
        let fixture = Fixture::with(git(BRANCH));
        let shown = RefCell::new(None);
        // The project is given by its number, and the user sees what that number is.
        let done = fixture.confirm(&ID.to_string(), &|target| {
            shown.replace(Some(target.clone()));
            Ok(format!("  {NAME}\n"))
        });
        assert_eq!(done.unwrap(), Done::Confirmed(NAME.to_owned()));
        assert_eq!(
            shown.take(),
            Some(Target {
                name: NAME.to_owned(),
                place: REPOSITORY.to_owned(),
                branch: Some(BRANCH.to_owned()),
            })
        );
        assert!(fixture.consent().confirmed);
    }

    #[test]
    fn a_folder_has_no_branch_and_any_other_answer_changes_nothing() {
        let fixture = Fixture::with(folder(FOLDER));
        let shown = RefCell::new(None);
        for answer in ["yes", "ja", "y", "", "SHOP", "shop2"] {
            let done = fixture.confirm(&ID.to_string(), &|target| {
                shown.replace(Some(target.clone()));
                Ok(answer.to_owned())
            });
            assert_eq!(done.unwrap(), Done::Declined(NAME.to_owned()), "{answer}");
        }
        assert_eq!(shown.take().map(|target| target.branch), Some(None));
        assert_eq!(fixture.consent(), ConsentRecord::default());
    }

    #[test]
    fn nothing_is_stored_if_the_project_changed_while_the_user_was_asked() {
        let fixture = Fixture::with(folder(FOLDER));
        let moved = fixture.confirm(&ID.to_string(), &|_| {
            fixture
                .store()
                .save_project(&project(folder(OTHER_FOLDER)))
                .unwrap();
            Ok(NAME.to_owned())
        });
        assert_eq!(moved.unwrap_err().code(), ErrorCode::IndexerNotPermitted);
        assert_eq!(fixture.consent(), ConsentRecord::default());
        let other = Fixture::with(git(BRANCH));
        let switched = other.confirm(&ID.to_string(), &|_| {
            other.store().save_project(&project(git("main"))).unwrap();
            Ok(NAME.to_owned())
        });
        assert_eq!(switched.unwrap_err().code(), ErrorCode::IndexerNotPermitted);
        assert_eq!(other.consent(), ConsentRecord::default());
        assert!(other.root.path().exists());
    }

    #[test]
    fn a_name_with_control_or_direction_characters_is_not_shown_and_nobody_is_asked() {
        for name in [
            "a\nb",
            "a\u{1b}[2Jb",
            "ab\u{202E}cd",
            "a\u{2066}b",
            "a\u{200B}b",
        ] {
            let fixture = Fixture::with(folder(FOLDER));
            let mut renamed = project(folder(FOLDER));
            renamed.name = ProjectName::new(name).unwrap();
            fixture.store().save_project(&renamed).unwrap();
            let asked = RefCell::new(false);
            let refused = fixture.confirm(&ID.to_string(), &|_| {
                asked.replace(true);
                Ok(name.to_owned())
            });
            assert_eq!(refused.unwrap_err().code(), ErrorCode::IndexerNotPermitted);
            assert!(!asked.take(), "asked for {name:?}");
            assert_eq!(fixture.consent(), ConsentRecord::default());
        }
    }
}
