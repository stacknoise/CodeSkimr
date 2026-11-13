//! The launch code and its exchange for the token (ADR 0028, ADR 0030).

mod common;

use std::time::Duration;

use common::{Call, Running, TOKEN, owner, start, start_with};

const ISSUE: &str = "/api/v1/session/launch-code";
const EXCHANGE: &str = "/api/v1/session/exchange";

#[cfg(test)]
async fn issue(server: &Running) -> String {
    let reply = Call::post(ISSUE).token(TOKEN).send(server).await;
    assert_eq!(reply.status, 200, "{}", reply.body);
    reply.json()["code"].as_str().unwrap().to_owned()
}

#[cfg(test)]
async fn exchange(server: &Running, code: &str) -> common::Reply {
    let body = serde_json::json!({ "code": code }).to_string();
    Call::post(EXCHANGE).json(&body).send(server).await
}

#[tokio::test]
async fn the_code_is_exchanged_once_for_the_token() {
    let server = start(owner()).await;
    let code = issue(&server).await;
    assert_eq!(code.len(), 32);
    let first = exchange(&server, &code).await;
    assert_eq!(first.status, 200, "{}", first.body);
    assert_eq!(first.json()["token"], TOKEN);
    assert_eq!(first.header("cache-control"), Some("no-store"));
    let second = exchange(&server, &code).await;
    assert_eq!(second.status, 401);
    assert_eq!(second.code(), "API-403");
    assert!(!second.body.contains(TOKEN));
}

#[tokio::test]
async fn the_issued_token_works_for_a_change() {
    let server = start(owner()).await;
    let code = issue(&server).await;
    let token = exchange(&server, &code).await.json()["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let reply = Call::post("/api/v1/commands/test.wipe")
        .token(&token)
        .send(&server)
        .await;
    assert_eq!(reply.status, 200);
}

#[tokio::test]
async fn a_new_code_voids_the_old_one_and_a_wrong_one_changes_nothing() {
    let server = start(owner()).await;
    let old = issue(&server).await;
    let new = issue(&server).await;
    assert_eq!(exchange(&server, &old).await.code(), "API-403");
    assert_eq!(exchange(&server, &"0".repeat(32)).await.code(), "API-403");
    assert_eq!(exchange(&server, "").await.code(), "API-403");
    assert_eq!(exchange(&server, &new).await.status, 200);
}

#[tokio::test]
async fn an_expired_code_gives_nothing() {
    let server = start_with(owner(), None, Duration::from_millis(1)).await;
    let code = issue(&server).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(exchange(&server, &code).await.code(), "API-403");
}

#[tokio::test]
async fn a_code_needs_the_token_to_be_created() {
    let server = start(owner()).await;
    for call in [
        Call::post(ISSUE),
        Call::post(ISSUE).token("csk_wrong"),
        Call::get(ISSUE),
    ] {
        let reply = call.send(&server).await;
        assert_eq!(reply.status, 401);
        assert_eq!(reply.code(), "API-401");
    }
}

#[tokio::test]
async fn the_state_shows_whether_a_code_waits() {
    let server = start(owner()).await;
    let pending = |reply: common::Reply| reply.json()["pending"].as_bool().unwrap();
    assert!(!pending(Call::get(ISSUE).token(TOKEN).send(&server).await));
    let code = issue(&server).await;
    assert!(pending(Call::get(ISSUE).token(TOKEN).send(&server).await));
    exchange(&server, &code).await;
    assert!(!pending(Call::get(ISSUE).token(TOKEN).send(&server).await));
}

#[tokio::test]
async fn host_and_origin_are_checked_on_both_routes() {
    let server = start(owner()).await;
    let code = issue(&server).await;
    let body = serde_json::json!({ "code": code }).to_string();
    for call in [
        Call::post(EXCHANGE).json(&body).host("evil.example"),
        Call::post(EXCHANGE)
            .json(&body)
            .header("Origin", "http://evil.example"),
        Call::post(EXCHANGE).json(&body).header("Origin", "null"),
        Call::post(ISSUE).token(TOKEN).host("evil.example"),
        Call::post(ISSUE)
            .token(TOKEN)
            .header("Origin", "http://evil.example"),
    ] {
        let reply = call.send(&server).await;
        assert_eq!(reply.status, 403);
        assert_eq!(reply.code(), "API-402");
    }
    // None of this has used the code.
    assert_eq!(exchange(&server, &code).await.status, 200);
}

#[tokio::test]
async fn the_exchange_takes_only_a_small_json_body() {
    let server = start(owner()).await;
    let code = issue(&server).await;
    let form = Call::post(EXCHANGE)
        .raw(
            "application/x-www-form-urlencoded",
            format!("code={code}").as_bytes(),
        )
        .send(&server)
        .await;
    assert_eq!(form.code(), "API-101");
    let text = Call::post(EXCHANGE)
        .raw("text/plain", format!(r#"{{"code":"{code}"}}"#).as_bytes())
        .send(&server)
        .await;
    assert_eq!(text.code(), "API-101");
    let big = format!(r#"{{"code":"{}"}}"#, "a".repeat(1000));
    let reply = Call::post(EXCHANGE).json(&big).send(&server).await;
    assert_eq!(reply.status, 413);
    let empty = Call::post(EXCHANGE).send(&server).await;
    assert_eq!(empty.code(), "API-101");
    let wrong_shape = Call::post(EXCHANGE)
        .json(r#"{"other":1}"#)
        .send(&server)
        .await;
    assert_eq!(wrong_shape.code(), "API-101");
    assert_eq!(exchange(&server, &code).await.status, 200);
}

#[tokio::test]
async fn only_the_documented_methods_are_allowed() {
    let server = start(owner()).await;
    assert_eq!(Call::get(EXCHANGE).send(&server).await.status, 405);
    assert_eq!(Call::new("PUT", ISSUE).send(&server).await.status, 405);
}
