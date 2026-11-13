//! The MCP route: the token for every request, `Host` and `Origin` as everywhere (blueprint S-04).

mod common;

use common::{Call, TOKEN, owner, start, start_mcp};

const MCP: &str = "/mcp";
const BODY: &str = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;

#[tokio::test]
async fn a_call_with_the_token_reaches_the_service() {
    let server = start_mcp(owner()).await;
    let reply = Call::post(MCP).token(TOKEN).json(BODY).send(&server).await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "pong");
}

#[tokio::test]
async fn every_method_needs_the_token_not_only_those_that_change_something() {
    let server = start_mcp(owner()).await;
    for method in ["GET", "POST", "DELETE"] {
        let reply = Call::new(method, MCP).json(BODY).send(&server).await;
        assert_eq!(reply.status, 401, "{method}");
        assert_eq!(reply.code(), "API-401");
    }
}

#[tokio::test]
async fn a_wrong_or_malformed_token_is_refused_like_a_missing_one() {
    let server = start_mcp(owner()).await;
    let wrong = format!("{TOKEN}x");
    let calls = [
        Call::post(MCP).token(&wrong).json(BODY),
        Call::post(MCP).token("").json(BODY),
        Call::post(MCP).header("Authorization", TOKEN).json(BODY),
        Call::post(MCP)
            .header("Authorization", "Basic abc")
            .json(BODY),
    ];
    for call in calls {
        let reply = call.send(&server).await;
        assert_eq!(reply.status, 401);
        assert_eq!(reply.code(), "API-401");
        assert!(!reply.body.contains("pong"));
    }
}

#[tokio::test]
async fn a_foreign_host_or_origin_is_refused_even_with_the_token() {
    let server = start_mcp(owner()).await;
    let foreign_host = Call::post(MCP)
        .token(TOKEN)
        .host("evil.example")
        .json(BODY)
        .send(&server)
        .await;
    assert_eq!(foreign_host.status, 403);
    assert_eq!(foreign_host.code(), "API-402");
    let foreign_origin = Call::post(MCP)
        .token(TOKEN)
        .header("Origin", "http://evil.example")
        .json(BODY)
        .send(&server)
        .await;
    assert_eq!(foreign_origin.status, 403);
    assert_eq!(foreign_origin.code(), "API-402");
}

#[tokio::test]
async fn the_answer_carries_the_hardening_headers() {
    let server = start_mcp(owner()).await;
    let reply = Call::post(MCP).token(TOKEN).json(BODY).send(&server).await;
    assert_eq!(reply.header("x-content-type-options"), Some("nosniff"));
    assert!(reply.header("cache-control").is_some());
}

#[tokio::test]
async fn without_a_service_there_is_no_mcp_route() {
    let server = start(owner()).await;
    let reply = Call::post(MCP).token(TOKEN).json(BODY).send(&server).await;
    assert_eq!(reply.status, 404);
    assert_eq!(reply.code(), "API-201");
}
