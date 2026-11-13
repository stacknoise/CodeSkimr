//! The command routes: generated from the registry, ordered checks, no oracle.

mod common;

use codeskimr_domain::Principal;
use common::{Call, TOKEN, owner, reader, start};
use serde_json::json;

const WIPE: &str = "/api/v1/commands/test.wipe";
const ECHO: &str = "/api/v1/commands/test.echo";

#[tokio::test]
async fn a_read_only_command_runs_without_a_token() {
    let server = start(owner()).await;
    let reply = Call::post(ECHO).json(r#""hi""#).send(&server).await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.json(), json!("HI"));
}

#[tokio::test]
async fn a_command_that_changes_something_needs_the_token() {
    let server = start(owner()).await;
    let cases = [
        Call::post(WIPE),
        Call::post(WIPE).token("csk_wrong"),
        Call::post(WIPE).header("Authorization", TOKEN),
        Call::post(WIPE).header("Authorization", "Basic abc"),
        Call::post(WIPE).token(""),
    ];
    for call in cases {
        let reply = call.send(&server).await;
        assert_eq!(reply.status, 401);
        assert_eq!(reply.code(), "API-401");
    }
    assert_eq!(server.wipes.load(std::sync::atomic::Ordering::SeqCst), 0);
    let reply = Call::post(WIPE).token(TOKEN).send(&server).await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.json(), json!(1));
}

#[tokio::test]
async fn the_token_is_never_echoed() {
    let server = start(owner()).await;
    let mut bodies = Vec::new();
    for call in [
        Call::post(WIPE).token("csk_wrong"),
        Call::post(WIPE).token(TOKEN),
        Call::get("/api/v1/commands").token(TOKEN),
        Call::post("/api/v1/commands/unknown").token(TOKEN),
    ] {
        let reply = call.send(&server).await;
        bodies.push(format!("{:?} {}", reply.headers, reply.body));
    }
    assert!(bodies.iter().all(|text| !text.contains(TOKEN)));
    assert!(bodies.iter().all(|text| !text.contains("csk_wrong")));
}

#[cfg(test)]
async fn listed_ids(principal: Principal) -> Vec<String> {
    let server = start(principal).await;
    let reply = Call::get("/api/v1/commands").send(&server).await;
    let entries = reply.json();
    entries
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn the_list_shows_what_the_caller_may_run() {
    assert_eq!(
        listed_ids(owner()).await,
        ["test.echo", "test.explode", "test.missing", "test.wipe"]
    );
    assert_eq!(
        listed_ids(reader()).await,
        ["test.echo", "test.explode", "test.missing"]
    );
}

#[tokio::test]
async fn a_caller_without_the_permission_is_denied_before_the_input_is_read() {
    let server = start(reader()).await;
    // Valid token, so the token check passes; the permission is what fails.
    let bodies = [
        Call::post(WIPE).token(TOKEN),
        Call::post(WIPE).token(TOKEN).json("{broken"),
        Call::post(WIPE).token(TOKEN).raw("text/plain", b"anything"),
        Call::post(WIPE)
            .token(TOKEN)
            .json(&"x".repeat(2 * 1024 * 1024)),
    ];
    let mut answers = Vec::new();
    for call in bodies {
        let reply = call.send(&server).await;
        assert_eq!(reply.status, 403);
        assert_eq!(reply.code(), "AUTH-401");
        answers.push(reply.body);
    }
    assert!(answers.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(server.wipes.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[tokio::test]
async fn unknown_commands_and_paths_have_a_code() {
    let server = start(owner()).await;
    for call in [
        Call::post("/api/v1/commands/nope"),
        Call::get("/api/v1/unknown"),
        Call::get("/"),
    ] {
        let reply = call.send(&server).await;
        assert_eq!(reply.status, 404);
        assert_eq!(reply.code(), "API-201");
    }
    let reply = Call::get(ECHO).send(&server).await;
    assert_eq!(reply.status, 405);
    assert_eq!(reply.code(), "API-101");
}

#[tokio::test]
async fn a_bad_body_is_refused_with_a_code() {
    let server = start(owner()).await;
    let broken = Call::post(ECHO).json("{broken").send(&server).await;
    assert_eq!((broken.status, broken.code().as_str()), (400, "API-101"));
    let wrong_shape = Call::post(ECHO).json(r#"{"other":1}"#).send(&server).await;
    assert_eq!(
        (wrong_shape.status, wrong_shape.code().as_str()),
        (400, "API-101")
    );
    let form = Call::post(ECHO)
        .raw("application/x-www-form-urlencoded", b"text=hi")
        .send(&server)
        .await;
    assert_eq!((form.status, form.code().as_str()), (400, "API-101"));
    let lookalike = Call::post(ECHO)
        .raw("application/jsonx", br#""hi""#)
        .send(&server)
        .await;
    assert_eq!(
        (lookalike.status, lookalike.code().as_str()),
        (400, "API-101")
    );
    let spelled = Call::post(ECHO)
        .raw("Application/JSON; charset=utf-8", br#""hi""#)
        .send(&server)
        .await;
    assert_eq!(spelled.status, 200);
    let plain = Call::post(ECHO)
        .raw("text/plain", br#""hi""#)
        .send(&server)
        .await;
    assert_eq!((plain.status, plain.code().as_str()), (400, "API-101"));
    let huge = Call::post(ECHO)
        .json(&" ".repeat(2 * 1024 * 1024))
        .send(&server)
        .await;
    assert_eq!((huge.status, huge.code().as_str()), (413, "API-501"));
}

#[tokio::test]
async fn a_failing_command_reports_its_own_code_and_nothing_else() {
    let server = start(owner()).await;
    let reply = Call::post("/api/v1/commands/test.missing")
        .send(&server)
        .await;
    assert_eq!((reply.status, reply.code().as_str()), (404, "PRJ-201"));
    assert_eq!(reply.json().as_object().unwrap().len(), 2);
}

#[tokio::test]
async fn a_panicking_command_gives_an_internal_error_without_its_text() {
    let server = start(owner()).await;
    let reply = Call::post("/api/v1/commands/test.explode")
        .send(&server)
        .await;
    assert_eq!((reply.status, reply.code().as_str()), (500, "APP-901"));
    assert!(!reply.body.contains("canary-panic-text"));
    // The server keeps serving.
    let again = Call::post(ECHO).json(r#""still""#).send(&server).await;
    assert_eq!(again.status, 200);
}
