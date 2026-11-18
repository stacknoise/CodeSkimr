//! The route that opens the console window of the level 2 confirmation (ADR 0071). Every step of
//! the guard has its own refusal here, and the window is a fake: nothing starts a process.
#![cfg(test)]

mod common;

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::level2::{ConfirmationDesk, DeskParts, Launch};
use codeskimr_application::ports::{
    ConfirmationWindow, ConsentGrant, ConsentRecord, ConsentStore, Level2Switch, PreviousIndex,
    ProjectRepository, WindowOutcome,
};
use codeskimr_application::registry::Registry;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    FolderPath, Permission, PermissionSet, Principal, Project, ProjectId, ProjectName,
    ProjectSource, SourceFile,
};
use codeskimr_server::{CommandApi, InstallationToken, Server};
use common::{Call, Reply, Running, TOKEN, owner, reader, ui_options};

const PATH: &str = "/api/v1/level2/confirm-window";
const KNOWN: u64 = 7;
const OTHER: u64 = 8;

#[derive(Default)]
struct Window {
    asked: Mutex<Vec<u64>>,
    answer: Mutex<Option<Result<WindowOutcome, ErrorCode>>>,
}

impl ConfirmationWindow for Window {
    fn open(&self, project: ProjectId) -> Result<WindowOutcome, CodedError> {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(project.get());
        let answer = *self.answer.lock().unwrap_or_else(PoisonError::into_inner);
        answer
            .unwrap_or(Ok(WindowOutcome::Opened))
            .map_err(CodedError::new)
    }
}

impl Window {
    fn asked(&self) -> Vec<u64> {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

struct Switch(bool);

impl Level2Switch for Switch {
    fn enabled(&self) -> Result<bool, CodedError> {
        Ok(self.0)
    }
}

struct Projects;

impl ProjectRepository for Projects {
    fn project(&self, id: ProjectId) -> Result<Project, CodedError> {
        if id.get() != KNOWN && id.get() != OTHER {
            return Err(CodedError::new(ErrorCode::ProjectNotFound));
        }
        Ok(Project {
            id,
            name: ProjectName::new("shop").unwrap(),
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

/// A store that refuses to be written: opening a window must never reach it.
struct ReadOnlyConsents;

impl ConsentStore for ReadOnlyConsents {
    fn consent(&self, _project: ProjectId) -> Result<ConsentRecord, CodedError> {
        Ok(ConsentRecord::default())
    }

    fn store_consent(&self, _grant: &ConsentGrant) -> Result<(), CodedError> {
        panic!("the route must never write a consent");
    }
}

struct Setup {
    server: Running,
    window: Arc<Window>,
}

fn desk(window: &Arc<Window>, switch: bool, launch: Launch, locked: bool) -> Arc<ConfirmationDesk> {
    Arc::new(ConfirmationDesk::new(DeskParts {
        window: window.clone(),
        switch: Arc::new(Switch(switch)),
        projects: Arc::new(Projects),
        consents: Arc::new(ReadOnlyConsents),
        launch,
        locked,
        cooldown: Duration::from_secs(3600),
    }))
}

async fn start_with(
    principal: Principal,
    desk: Option<(Arc<Window>, Arc<ConfirmationDesk>)>,
) -> Setup {
    let api = CommandApi::new(Arc::new(Registry::new()));
    let events = EventBus::new();
    let mut server = Server::new(
        api,
        events.clone(),
        principal,
        InstallationToken::from_token(TOKEN),
        ui_options(),
    );
    let window = desk
        .as_ref()
        .map_or_else(Arc::default, |(w, _)| Arc::clone(w));
    if let Some((_, desk)) = desk {
        server = server.with_confirmation(desk);
    }
    let bound = server.bind(([127, 0, 0, 1], 0).into()).await.unwrap();
    let address = bound.address().unwrap();
    tokio::spawn(bound.serve(std::future::pending()));
    let server = Running {
        address,
        wipes: Arc::default(),
        events,
    };
    Setup { server, window }
}

async fn start() -> Setup {
    let window = Arc::new(Window::default());
    let desk = desk(&window, true, Launch::Window, false);
    start_with(owner(), Some((window, desk))).await
}

fn request(id: u64) -> Call<'static> {
    Call::post(PATH)
        .token(TOKEN)
        .json(&format!("{{\"project_id\":{id}}}"))
}

async fn refused(setup: &Setup, call: Call<'_>, status: u16, code: &str) -> Reply {
    let reply = call.send(&setup.server).await;
    assert_eq!(reply.status, status, "{}", reply.body);
    assert_eq!(reply.code(), code, "{}", reply.body);
    assert!(setup.window.asked().is_empty(), "a window was asked for");
    reply
}

#[tokio::test]
async fn a_click_of_the_page_opens_one_window_for_the_id_and_answers_window() {
    let setup = start().await;
    let reply = request(KNOWN)
        .header("Origin", &format!("http://{}", setup.server.address))
        .header("Sec-Fetch-Site", "same-origin")
        .send(&setup.server)
        .await;
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(reply.json(), serde_json::json!({ "launch": "window" }));
    assert_eq!(setup.window.asked(), [KNOWN]);
    assert_eq!(reply.header("cache-control"), Some("no-store"));
}

#[tokio::test]
async fn a_system_without_a_window_gets_the_command_with_the_id() {
    let window = Arc::new(Window::default());
    *window.answer.lock().unwrap() = Some(Ok(WindowOutcome::Unavailable));
    let desk = desk(&window, true, Launch::Window, false);
    let setup = start_with(owner(), Some((window, desk))).await;
    let reply = request(KNOWN).send(&setup.server).await;
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(
        reply.json(),
        serde_json::json!({ "launch": "copy", "command": "codeskimr level2 confirm 7" })
    );
}

#[tokio::test]
async fn a_foreign_host_or_origin_gets_api_402_and_no_window() {
    let setup = start().await;
    refused(&setup, request(KNOWN).host("evil.example"), 403, "API-402").await;
    refused(
        &setup,
        request(KNOWN).header("Origin", "http://evil.example"),
        403,
        "API-402",
    )
    .await;
    refused(
        &setup,
        request(KNOWN).header("Origin", "null"),
        403,
        "API-402",
    )
    .await;
}

#[tokio::test]
async fn a_browser_that_says_the_request_is_from_another_site_gets_api_402() {
    let setup = start().await;
    for site in ["cross-site", "same-site", "bogus", ""] {
        refused(
            &setup,
            request(KNOWN).header("Sec-Fetch-Site", site),
            403,
            "API-402",
        )
        .await;
    }
    // The two own values pass.
    for site in ["same-origin", "none"] {
        let reply = request(KNOWN)
            .header("Sec-Fetch-Site", site)
            .send(&setup.server)
            .await;
        assert_ne!(reply.status, 403, "{site}");
    }
}

#[tokio::test]
async fn a_body_that_is_not_json_gets_api_101_before_the_token_is_looked_at() {
    let setup = start().await;
    let body = b"project_id=7";
    for media in [
        "text/plain",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
    ] {
        let call = Call::post(PATH).token(TOKEN).raw(media, body);
        refused(&setup, call, 400, "API-101").await;
    }
    // A form without a token: the content type is the first refusal.
    let call = Call::post(PATH).raw("text/plain", body);
    refused(&setup, call, 400, "API-101").await;
    refused(&setup, Call::post(PATH).token(TOKEN), 400, "API-101").await;
}

#[tokio::test]
async fn a_missing_or_wrong_token_gets_api_401_and_no_window() {
    let setup = start().await;
    let body = format!("{{\"project_id\":{KNOWN}}}");
    refused(&setup, Call::post(PATH).json(&body), 401, "API-401").await;
    refused(
        &setup,
        Call::post(PATH).token("csk_wrong").json(&body),
        401,
        "API-401",
    )
    .await;
    refused(
        &setup,
        Call::post(PATH).token("").json(&body),
        401,
        "API-401",
    )
    .await;
}

#[tokio::test]
async fn a_caller_without_the_right_to_open_gets_auth_401() {
    let window = Arc::new(Window::default());
    let desk = desk(&window, true, Launch::Window, false);
    // Everything but the one right, and a reader.
    let all_but: PermissionSet = Permission::ALL
        .iter()
        .copied()
        .filter(|p| *p != Permission::OpenConfirmation)
        .collect();
    for principal in [Principal::limited(all_but), reader()] {
        let setup = start_with(principal, Some((window.clone(), desk.clone()))).await;
        refused(&setup, request(KNOWN), 403, "AUTH-401").await;
    }
}

#[tokio::test]
async fn a_broken_body_gets_api_101() {
    let setup = start().await;
    for body in [
        "",
        "{",
        "[]",
        "{\"project_id\":\"7\"}",
        "{\"project_id\":-1}",
        "{\"project_id\":7,\"name\":\"x\"}",
        "{}",
    ] {
        refused(
            &setup,
            Call::post(PATH).token(TOKEN).json(body),
            400,
            "API-101",
        )
        .await;
    }
    let big = format!("{{\"project_id\":7,\"pad\":\"{}\"}}", "x".repeat(1000));
    refused(
        &setup,
        Call::post(PATH).token(TOKEN).json(&big),
        413,
        "API-501",
    )
    .await;
}

#[tokio::test]
async fn a_second_click_inside_the_pause_gets_api_504_with_retry_after_and_no_window() {
    let setup = start().await;
    assert_eq!(request(KNOWN).send(&setup.server).await.status, 200);
    let second = request(KNOWN).send(&setup.server).await;
    assert_eq!(second.status, 429, "{}", second.body);
    assert_eq!(second.code(), "API-504");
    let wait: u64 = second.header("retry-after").unwrap().parse().unwrap();
    assert!((1..=3600).contains(&wait), "{wait}");
    // Another project, same pause.
    assert_eq!(request(OTHER).send(&setup.server).await.code(), "API-504");
    assert_eq!(setup.window.asked(), [KNOWN]);
}

#[tokio::test]
async fn an_unknown_project_the_switch_and_a_team_server_each_have_their_own_answer() {
    let window = Arc::new(Window::default());
    let setup = start_with(
        owner(),
        Some((window.clone(), desk(&window, true, Launch::Window, false))),
    )
    .await;
    refused(&setup, request(99), 404, "PRJ-201").await;
    let window = Arc::new(Window::default());
    let off = start_with(
        owner(),
        Some((window.clone(), desk(&window, false, Launch::Window, false))),
    )
    .await;
    refused(&off, request(KNOWN), 409, "CFG-105").await;
    let window = Arc::new(Window::default());
    let team = start_with(
        owner(),
        Some((window.clone(), desk(&window, true, Launch::Window, true))),
    )
    .await;
    refused(&team, request(KNOWN), 403, "SEC-403").await;
}

#[tokio::test]
async fn a_window_that_cannot_start_is_app_604() {
    let window = Arc::new(Window::default());
    *window.answer.lock().unwrap() = Some(Err(ErrorCode::ConfirmWindowFailed));
    let desk = desk(&window, true, Launch::Window, false);
    let setup = start_with(owner(), Some((window.clone(), desk))).await;
    let reply = request(KNOWN).send(&setup.server).await;
    assert_eq!(reply.status, 500, "{}", reply.body);
    assert_eq!(reply.code(), "APP-604");
}

#[tokio::test]
async fn a_server_without_a_desk_has_no_such_route_and_it_is_no_registry_command() {
    let setup = start_with(owner(), None).await;
    let reply = request(KNOWN).send(&setup.server).await;
    assert_eq!(reply.code(), "API-201");
    let setup = start().await;
    let listed = Call::get("/api/v1/commands").send(&setup.server).await;
    assert!(!listed.body.contains("level2"), "{}", listed.body);
    let as_command = Call::post("/api/v1/commands/level2.confirm-window")
        .token(TOKEN)
        .json("{\"project_id\":7}")
        .send(&setup.server)
        .await;
    assert_eq!(as_command.code(), "API-201");
    let get = Call::get(PATH).token(TOKEN).send(&setup.server).await;
    assert_eq!(get.status, 405);
    assert!(setup.window.asked().is_empty());
}
