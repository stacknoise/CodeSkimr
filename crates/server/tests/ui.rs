//! The UI files, the content security policy and the settings route (ADR 0028, ADR 0030).

mod common;

use std::fs;
use std::time::Duration;

use common::{Call, owner, start, start_with};

#[cfg(test)]
fn ui_folder() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("index.html"),
        "<!doctype html><title>x</title>",
    )
    .unwrap();
    fs::create_dir(dir.path().join("assets")).unwrap();
    fs::write(dir.path().join("assets").join("app-1.js"), "export {}").unwrap();
    fs::write(dir.path().join("assets").join("app-1.css"), "a{}").unwrap();
    fs::write(dir.path().join("secret.txt"), "outside").unwrap();
    dir
}

#[tokio::test]
async fn the_ui_is_served_with_the_right_media_types() {
    let dir = ui_folder();
    let server = start_with(
        owner(),
        Some(dir.path().to_path_buf()),
        Duration::from_secs(60),
    )
    .await;
    let page = Call::get("/").send(&server).await;
    assert_eq!(page.status, 200);
    assert_eq!(
        page.header("content-type"),
        Some("text/html; charset=utf-8")
    );
    assert!(page.body.contains("<title>"));
    let script = Call::get("/assets/app-1.js").send(&server).await;
    assert_eq!(
        script.header("content-type"),
        Some("text/javascript; charset=utf-8")
    );
    let style = Call::get("/assets/app-1.css").send(&server).await;
    assert_eq!(
        style.header("content-type"),
        Some("text/css; charset=utf-8")
    );
    let route = Call::get("/settings").send(&server).await;
    assert_eq!(route.status, 200);
    assert!(route.body.contains("<title>"));
}

#[tokio::test]
async fn unknown_files_and_api_paths_are_not_pages() {
    let dir = ui_folder();
    let server = start_with(
        owner(),
        Some(dir.path().to_path_buf()),
        Duration::from_secs(60),
    )
    .await;
    for path in ["/assets/missing.js", "/api/v1/unknown", "/api/", "/assets"] {
        let reply = Call::get(path).send(&server).await;
        // `/assets` has no extension and falls back to the start page.
        if path == "/assets" {
            assert_eq!(reply.status, 200);
            continue;
        }
        assert_eq!(reply.status, 404, "{path}");
        assert_eq!(reply.code(), "API-201");
    }
    let post = Call::post("/settings").json("{}").send(&server).await;
    assert_eq!(post.code(), "API-201");
}

#[tokio::test]
async fn paths_cannot_leave_the_folder() {
    let dir = ui_folder();
    let server = start_with(
        owner(),
        Some(dir.path().join("assets")),
        Duration::from_secs(60),
    )
    .await;
    for path in [
        "/../secret.txt",
        "/%2e%2e/secret.txt",
        "/..%2fsecret.txt",
        "/assets/../../secret.txt",
        "/.hidden",
        "/a%5cb.js",
    ] {
        let reply = Call::get(path).send(&server).await;
        assert_eq!(reply.status, 404, "{path}");
        assert!(!reply.body.contains("outside"), "{path}");
    }
}

#[tokio::test]
async fn without_a_ui_folder_every_unknown_path_is_an_api_error() {
    let server = start(owner()).await;
    let reply = Call::get("/").send(&server).await;
    assert_eq!(reply.status, 404);
    assert_eq!(reply.code(), "API-201");
}

#[tokio::test]
async fn every_answer_carries_the_policy() {
    let dir = ui_folder();
    let server = start_with(
        owner(),
        Some(dir.path().to_path_buf()),
        Duration::from_secs(60),
    )
    .await;
    let port = server.address.port();
    let calls = [
        Call::get("/"),
        Call::get("/assets/app-1.js"),
        Call::get("/api/v1/commands"),
        Call::get("/nothing.js"),
        Call::get("/").host("evil.example"),
    ];
    for call in calls {
        let reply = call.send(&server).await;
        let policy = reply.header("content-security-policy").unwrap();
        assert!(policy.starts_with("default-src 'self'"), "{policy}");
        for directive in [
            "frame-ancestors 'none'",
            "base-uri 'none'",
            "object-src 'none'",
        ] {
            assert!(policy.contains(directive), "{policy}");
        }
        assert!(
            policy.contains(&format!("ws://127.0.0.1:{port}")),
            "{policy}"
        );
        assert!(!policy.contains("unsafe-"), "{policy}");
        assert!(!policy.contains('*'), "{policy}");
        assert_eq!(reply.header("referrer-policy"), Some("no-referrer"));
        assert_eq!(reply.header("x-content-type-options"), Some("nosniff"));
    }
}

#[tokio::test]
async fn the_settings_are_public_and_hold_no_secret() {
    let server = start(owner()).await;
    let reply = Call::get("/api/v1/ui-config").send(&server).await;
    assert_eq!(reply.status, 200);
    let json = reply.json();
    assert_eq!(json["default_theme"], "petrol");
    assert_eq!(
        json["editor"]["url_template"],
        "vscode://file/{path}:{line}"
    );
    assert_eq!(json["editor"]["allowed_schemes"][0], "vscode");
    let foreign = Call::get("/api/v1/ui-config")
        .host("evil.example")
        .send(&server)
        .await;
    assert_eq!(foreign.status, 403);
}
