//! The state of level 2 per project and the request for the confirmation window (ADR 0071). The
//! window is a fake: nothing here starts a process.

#[cfg(test)]
mod common;

#[cfg(test)]
mod cases {
    use super::common::InMemoryConsent;
    use std::sync::{Arc, Mutex, PoisonError};
    use std::time::Duration;

    use codeskimr_application::error::CodedError;
    use codeskimr_application::level2::{
        ConfirmationDesk, DeskParts, Launch, Opened, State, open_confirmation, status,
    };
    use codeskimr_application::ports::{
        ConfirmationWindow, ConsentRecord, ConsentStore, Level2Switch, PreviousIndex,
        ProjectRepository, WindowOutcome,
    };
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{
        FolderPath, Permission, PermissionSet, Principal, Project, ProjectId, ProjectName,
        ProjectSource, SourceFile,
    };

    const KNOWN: ProjectId = ProjectId::new(7);
    const OTHER: ProjectId = ProjectId::new(8);
    const NAME: &str = "shop";
    const PAUSE: Duration = Duration::from_secs(3600);

    /// A window that records which projects it was asked for and answers what the test says.
    struct FakeWindow {
        asked: Mutex<Vec<u64>>,
        answer: Mutex<Result<WindowOutcome, ErrorCode>>,
    }

    impl FakeWindow {
        fn answering(answer: Result<WindowOutcome, ErrorCode>) -> Arc<Self> {
            Arc::new(Self {
                asked: Mutex::new(Vec::new()),
                answer: Mutex::new(answer),
            })
        }

        fn asked(&self) -> Vec<u64> {
            self.asked
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        }

        fn answer(&self, answer: Result<WindowOutcome, ErrorCode>) {
            *self.answer.lock().unwrap_or_else(PoisonError::into_inner) = answer;
        }
    }

    impl ConfirmationWindow for FakeWindow {
        fn open(&self, project: ProjectId) -> Result<WindowOutcome, CodedError> {
            self.asked
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(project.get());
            (*self.answer.lock().unwrap_or_else(PoisonError::into_inner)).map_err(CodedError::new)
        }
    }

    struct FakeSwitch(Mutex<bool>);

    impl Level2Switch for FakeSwitch {
        fn enabled(&self) -> Result<bool, CodedError> {
            Ok(*self.0.lock().unwrap_or_else(PoisonError::into_inner))
        }
    }

    /// One project, with the name the test chooses.
    struct Projects(String);

    impl ProjectRepository for Projects {
        fn project(&self, id: ProjectId) -> Result<Project, CodedError> {
            if id != KNOWN && id != OTHER {
                return Err(CodedError::new(ErrorCode::ProjectNotFound));
            }
            Ok(Project {
                id,
                name: ProjectName::new(self.0.clone()).unwrap(),
                source: ProjectSource::Folder(FolderPath::new("/work/shop").unwrap()),
                layers: Vec::new(),
            })
        }

        fn latest_files(&self, _id: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
            Ok(Vec::new())
        }

        fn latest_index(&self, _id: ProjectId) -> Result<PreviousIndex, CodedError> {
            Ok(PreviousIndex::default())
        }
    }

    struct Fixture {
        window: Arc<FakeWindow>,
        switch: Arc<FakeSwitch>,
        consent: Arc<InMemoryConsent>,
        desk: ConfirmationDesk,
    }

    fn fixture_named(name: &str, launch: Launch, locked: bool, cooldown: Duration) -> Fixture {
        let window = FakeWindow::answering(Ok(WindowOutcome::Opened));
        let switch = Arc::new(FakeSwitch(Mutex::new(true)));
        let consent = Arc::new(InMemoryConsent::default());
        let desk = ConfirmationDesk::new(DeskParts {
            window: window.clone(),
            switch: switch.clone(),
            projects: Arc::new(Projects(name.to_owned())),
            consents: consent.clone(),
            launch,
            locked,
            cooldown,
        });
        Fixture {
            window,
            switch,
            consent,
            desk,
        }
    }

    fn fixture() -> Fixture {
        fixture_named(NAME, Launch::Window, false, PAUSE)
    }

    /// What a REST client of the local server holds for this feature, and nothing that confirms.
    fn client() -> Principal {
        Principal::limited(PermissionSet::from_iter([
            Permission::Read,
            Permission::Scan,
            Permission::Manage,
            Permission::Admin,
            Permission::RevokeIndexer,
            Permission::OpenConfirmation,
        ]))
    }

    fn code<T: std::fmt::Debug>(result: Result<T, CodedError>) -> ErrorCode {
        result.unwrap_err().code()
    }

    fn confirmed() -> ConsentRecord {
        ConsentRecord {
            enabled: true,
            confirmed: true,
        }
    }

    fn state_of(fixture: &Fixture) -> (State, bool) {
        let answer = status(&fixture.desk, KNOWN).unwrap();
        assert_eq!(answer.project, KNOWN.get());
        (answer.state, answer.confirmed)
    }

    #[test]
    fn the_state_follows_the_switch_and_the_confirmation() {
        let fixture = fixture();
        assert_eq!(state_of(&fixture), (State::Off, false));
        fixture.consent.set(confirmed());
        assert_eq!(state_of(&fixture), (State::On, true));
        *fixture.switch.0.lock().unwrap() = false;
        // A confirmation that exists is reported even though the switch is off.
        assert_eq!(state_of(&fixture), (State::NotEnabled, true));
        fixture.consent.set(ConsentRecord::default());
        assert_eq!(state_of(&fixture), (State::NotEnabled, false));
    }

    #[test]
    fn a_team_server_is_locked_and_the_state_says_so() {
        let fixture = fixture_named(NAME, Launch::Window, true, PAUSE);
        assert_eq!(state_of(&fixture).0, State::Locked);
        assert_eq!(
            code(open_confirmation(&client(), &fixture.desk, KNOWN)),
            ErrorCode::IndexerNotPermitted
        );
        assert!(fixture.window.asked().is_empty());
    }

    #[test]
    fn the_state_names_how_the_confirmation_is_reached_and_an_unknown_project_is_prj_201() {
        let fixture = fixture_named(NAME, Launch::Copy, false, PAUSE);
        let answer = status(&fixture.desk, KNOWN).unwrap();
        assert_eq!(answer.launch, Launch::Copy);
        assert_eq!(
            code(status(&fixture.desk, ProjectId::new(99))),
            ErrorCode::ProjectNotFound
        );
    }

    #[test]
    fn a_click_opens_one_window_for_the_project_id_and_nothing_else() {
        let fixture = fixture();
        let opened = open_confirmation(&client(), &fixture.desk, KNOWN).unwrap();
        assert_eq!(opened, Opened::Window);
        assert_eq!(fixture.window.asked(), [KNOWN.get()]);
        // Opening a window confirms nothing.
        assert_eq!(
            fixture.consent.consent(KNOWN).expect("record"),
            ConsentRecord::default()
        );
    }

    #[test]
    fn a_second_request_inside_the_pause_opens_nothing_for_any_project() {
        let fixture = fixture();
        open_confirmation(&client(), &fixture.desk, KNOWN).unwrap();
        for project in [KNOWN, OTHER] {
            assert_eq!(
                code(open_confirmation(&client(), &fixture.desk, project)),
                ErrorCode::ConfirmWindowRateLimited
            );
        }
        assert_eq!(fixture.window.asked(), [KNOWN.get()]);
        assert!(fixture.desk.retry_after() > Duration::ZERO);
        assert!(fixture.desk.retry_after() <= PAUSE);
    }

    #[test]
    fn without_a_pause_the_next_request_opens_the_next_window() {
        let fixture = fixture_named(NAME, Launch::Window, false, Duration::ZERO);
        open_confirmation(&client(), &fixture.desk, KNOWN).unwrap();
        open_confirmation(&client(), &fixture.desk, KNOWN).unwrap();
        assert_eq!(fixture.window.asked().len(), 2);
    }

    #[test]
    fn a_request_refused_before_the_launch_does_not_start_the_pause() {
        let fixture = fixture();
        assert_eq!(
            code(open_confirmation(
                &client(),
                &fixture.desk,
                ProjectId::new(99)
            )),
            ErrorCode::ProjectNotFound
        );
        *fixture.switch.0.lock().unwrap() = false;
        assert_eq!(
            code(open_confirmation(&client(), &fixture.desk, KNOWN)),
            ErrorCode::Level2SwitchOff
        );
        *fixture.switch.0.lock().unwrap() = true;
        fixture.window.answer(Ok(WindowOutcome::AlreadyOpen));
        assert_eq!(
            open_confirmation(&client(), &fixture.desk, KNOWN).unwrap(),
            Opened::Window
        );
        fixture.window.answer(Ok(WindowOutcome::Unavailable));
        assert!(matches!(
            open_confirmation(&client(), &fixture.desk, KNOWN).unwrap(),
            Opened::Copy { .. }
        ));
        fixture.window.answer(Ok(WindowOutcome::Opened));
        // None of the four requests started the pause, so this one still opens a window.
        open_confirmation(&client(), &fixture.desk, KNOWN).unwrap();
        assert_eq!(fixture.window.asked().len(), 3);
    }

    #[test]
    fn a_start_that_fails_keeps_the_pause() {
        let fixture = fixture();
        fixture.window.answer(Err(ErrorCode::ConfirmWindowFailed));
        assert_eq!(
            code(open_confirmation(&client(), &fixture.desk, KNOWN)),
            ErrorCode::ConfirmWindowFailed
        );
        assert_eq!(
            code(open_confirmation(&client(), &fixture.desk, KNOWN)),
            ErrorCode::ConfirmWindowRateLimited
        );
        assert_eq!(fixture.window.asked().len(), 1);
    }

    #[test]
    fn inside_the_pause_the_real_reason_of_a_refusal_comes_first() {
        let fixture = fixture();
        open_confirmation(&client(), &fixture.desk, KNOWN).unwrap();
        assert_eq!(
            code(open_confirmation(
                &client(),
                &fixture.desk,
                ProjectId::new(99)
            )),
            ErrorCode::ProjectNotFound
        );
        *fixture.switch.0.lock().unwrap() = false;
        assert_eq!(
            code(open_confirmation(&client(), &fixture.desk, KNOWN)),
            ErrorCode::Level2SwitchOff
        );
    }

    #[test]
    fn the_switch_of_the_configuration_comes_before_the_launch() {
        let fixture = fixture();
        *fixture.switch.0.lock().unwrap() = false;
        assert_eq!(
            code(open_confirmation(&client(), &fixture.desk, KNOWN)),
            ErrorCode::Level2SwitchOff
        );
        assert!(fixture.window.asked().is_empty());
    }

    #[test]
    fn a_system_without_a_window_answers_with_the_command_that_holds_the_id() {
        let command = Opened::Copy {
            command: "codeskimr level2 confirm 7".to_owned(),
        };
        let unavailable = fixture();
        unavailable.window.answer(Ok(WindowOutcome::Unavailable));
        assert_eq!(
            open_confirmation(&client(), &unavailable.desk, KNOWN).unwrap(),
            command
        );
        // A desk that knows its system has none never asks the port.
        let copy_only = fixture_named(NAME, Launch::Copy, false, PAUSE);
        assert_eq!(
            open_confirmation(&client(), &copy_only.desk, KNOWN).unwrap(),
            command
        );
        assert!(copy_only.window.asked().is_empty());
    }

    #[test]
    fn a_name_with_control_or_direction_characters_gets_no_window() {
        for name in [
            "shop\nrm",
            "shop\u{202E}gpj",
            "sh\u{200B}op",
            "sh\u{1b}[2Jop",
        ] {
            let fixture = fixture_named(name, Launch::Window, false, PAUSE);
            assert_eq!(
                code(open_confirmation(&client(), &fixture.desk, KNOWN)),
                ErrorCode::IndexerNotPermitted,
                "{name:?}"
            );
            assert!(fixture.window.asked().is_empty());
        }
    }

    #[test]
    fn the_right_to_open_is_checked_first_and_no_other_right_stands_in() {
        let fixture = fixture();
        let others: Vec<Permission> = Permission::ALL
            .iter()
            .copied()
            .filter(|p| *p != Permission::OpenConfirmation)
            .collect();
        let without = Principal::limited(others.into_iter().collect());
        assert_eq!(
            code(open_confirmation(&without, &fixture.desk, KNOWN)),
            ErrorCode::PermissionDenied
        );
        assert!(fixture.window.asked().is_empty());
    }
}
