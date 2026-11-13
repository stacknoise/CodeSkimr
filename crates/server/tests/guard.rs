//! Host, Origin, CORS and headers: S-04 against DNS rebinding and cross-site requests.

mod common;

use common::{Call, TOKEN, owner, start};

const ECHO: &str = "/api/v1/commands/test.echo";
const BODY: &str = r#"{"text":"hi"}"#;

#[tokio::test]
async fn the_address_and_localhost_with_the_right_port_are_accepted() {
    let server = start(owner()).await;
    let port = server.address.port();
    for host in [
        server.address.to_string(),
        format!("localhost:{port}"),
        format!("LocalHost:{port}"),
    ] {
        let reply = Call::post(ECHO).host(&host).json(BODY).send(&server).await;
        assert_eq!(reply.status, 200, "{host}");
    }
}

#[tokio::test]
async fn a_foreign_host_is_refused_whatever_the_route() {
    let server = start(owner()).await;
    let port = server.address.port();
    let hosts = [
        "evil.example".to_owned(),
        format!("evil.example:{port}"),
        // A rebound name resolves to 127.0.0.1 but keeps its own name in `Host`.
        format!("rebind.evil.example:{port}"),
        "127.0.0.1".to_owned(),
        format!("127.0.0.1:{}", port.wrapping_add(1)),
        format!("localhost.evil.example:{port}"),
        format!("localhost:{port}.evil.example"),
        String::new(),
    ];
    for host in hosts {
        for call in [
            Call::get("/api/v1/commands"),
            Call::post(ECHO).json(BODY),
            Call::get("/nothing"),
        ] {
            let reply = call.host(&host).send(&server).await;
            assert_eq!(reply.status, 403, "{host}");
            assert_eq!(reply.code(), "API-402");
        }
    }
}

#[tokio::test]
async fn a_foreign_origin_is_refused_and_an_own_one_is_not() {
    let server = start(owner()).await;
    let own = format!("http://{}", server.address);
    let localhost = format!("http://localhost:{}", server.address.port());
    for origin in [&own, &localhost] {
        let reply = Call::post(ECHO)
            .header("Origin", origin)
            .json(BODY)
            .send(&server)
            .await;
        assert_eq!(reply.status, 200, "{origin}");
    }
    for origin in [
        "http://evil.example",
        "https://evil.example",
        "null",
        "",
        "https://127.0.0.1",
        &format!("https://{}", server.address),
        &format!("http://{}.evil.example", server.address),
    ] {
        let reply = Call::post(ECHO)
            .header("Origin", origin)
            .json(BODY)
            .send(&server)
            .await;
        assert_eq!(reply.status, 403, "{origin}");
        assert_eq!(reply.code(), "API-402");
    }
}

#[tokio::test]
async fn a_foreign_origin_cannot_use_the_token() {
    let server = start(owner()).await;
    let reply = Call::post("/api/v1/commands/test.wipe")
        .header("Origin", "http://evil.example")
        .token(TOKEN)
        .send(&server)
        .await;
    assert_eq!((reply.status, reply.code().as_str()), (403, "API-402"));
    assert_eq!(server.wipes.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[tokio::test]
async fn cors_stays_closed() {
    let server = start(owner()).await;
    let own = format!("http://{}", server.address);
    let preflight = Call::new("OPTIONS", ECHO)
        .header("Origin", &own)
        .header("Access-Control-Request-Method", "POST")
        .send(&server)
        .await;
    assert!(preflight.status >= 400, "{}", preflight.status);
    let answers = [
        preflight,
        Call::post(ECHO)
            .header("Origin", &own)
            .json(BODY)
            .send(&server)
            .await,
        Call::get("/api/v1/commands").send(&server).await,
        Call::get("/nothing").send(&server).await,
    ];
    for reply in answers {
        assert!(
            reply
                .headers
                .iter()
                .all(|(name, _)| !name.to_ascii_lowercase().starts_with("access-control-")),
            "{:?}",
            reply.headers
        );
    }
}

#[tokio::test]
async fn every_answer_is_marked_not_to_be_cached_or_sniffed() {
    let server = start(owner()).await;
    let answers = [
        Call::get("/api/v1/commands").send(&server).await,
        Call::get("/nothing").send(&server).await,
        Call::get("/nothing")
            .host("evil.example")
            .send(&server)
            .await,
        Call::post(ECHO).json(BODY).send(&server).await,
    ];
    for reply in answers {
        assert_eq!(reply.header("cache-control"), Some("no-store"));
        assert_eq!(reply.header("x-content-type-options"), Some("nosniff"));
    }
}
