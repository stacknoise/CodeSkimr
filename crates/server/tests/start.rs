//! Starting: only loopback addresses (S-04), and a taken port is a coded failure.

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use codeskimr_application::events::EventBus;
use codeskimr_application::registry::Registry;
use codeskimr_codes::Coded;
use codeskimr_domain::{PermissionSet, Principal};
use codeskimr_server::{
    CommandApi, EditorSettings, InstallationToken, Server, UiOptions, UiSettings, ensure_loopback,
};

fn server() -> Server {
    Server::new(
        CommandApi::new(Arc::new(Registry::new())),
        EventBus::new(),
        Principal::limited(PermissionSet::all()),
        InstallationToken::from_token("csk_x"),
        UiOptions {
            dir: None,
            embedded: &[],
            launch_lifetime: std::time::Duration::from_secs(60),
            settings: UiSettings {
                editor: EditorSettings {
                    url_template: String::new(),
                    allowed_schemes: Vec::new(),
                },
                default_theme: String::new(),
            },
        },
    )
}

#[test]
fn only_loopback_addresses_pass() {
    for address in [
        SocketAddr::from((Ipv4Addr::LOCALHOST, 7421)),
        SocketAddr::from((Ipv4Addr::new(127, 9, 9, 9), 7421)),
        SocketAddr::from((Ipv6Addr::LOCALHOST, 7421)),
    ] {
        assert!(ensure_loopback(address).is_ok(), "{address}");
    }
    for address in [
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 7421)),
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 7421)),
        SocketAddr::from((Ipv4Addr::new(192, 168, 1, 5), 7421)),
        SocketAddr::from((Ipv4Addr::new(10, 0, 0, 1), 7421)),
        SocketAddr::from((Ipv4Addr::new(8, 8, 8, 8), 7421)),
        SocketAddr::from((Ipv4Addr::BROADCAST, 7421)),
        // An IPv4 address in IPv6 clothes is not accepted either: no guessing.
        "[::ffff:127.0.0.1]:7421".parse().unwrap(),
    ] {
        let error = ensure_loopback(address).expect_err(&address.to_string());
        assert_eq!(error.code().id(), "API-102", "{address}");
    }
}

#[tokio::test]
async fn a_non_loopback_address_never_opens_a_socket() {
    let error = server()
        .bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
        .await
        .err()
        .expect("must fail");
    assert_eq!(error.code().id(), "API-102");
}

#[tokio::test]
async fn a_taken_port_is_reported_with_its_code() {
    let first = server()
        .bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .unwrap();
    let taken = first.address().unwrap();
    let error = server().bind(taken).await.err().expect("must fail");
    assert_eq!(error.code().id(), "API-301");
}

#[tokio::test]
async fn the_server_stops_when_asked() {
    let bound = server()
        .bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(bound.serve(async move {
        stopped.await.ok();
    }));
    stop.send(()).unwrap();
    task.await.unwrap().unwrap();
}
