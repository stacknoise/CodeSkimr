//! The settings overview through the same path the server takes.

use serde_json::Value;

use super::tests::Fixture;

const USER_FILE: &str = "[scan]\nmax_workers = 2\n";

fn setting<'a>(shown: &'a Value, key: &str) -> &'a Value {
    shown["settings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["key"] == key)
        .unwrap()
}

#[test]
fn every_setting_is_shown_with_the_layer_that_set_it() {
    let fixture = Fixture::installed_with(None, Some(USER_FILE));
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(setting(&shown, "scan.max_workers")["value"], "2");
    assert_eq!(setting(&shown, "scan.max_workers")["origin"], "user");
    assert_eq!(setting(&shown, "server.port")["origin"], "default");
    assert_eq!(setting(&shown, "ui.dir")["origin"], "cli");
}

#[test]
fn the_files_and_flags_of_the_run_are_named() {
    let fixture = Fixture::installed_with(None, Some(USER_FILE));
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(shown["user_file"]["found"], true);
    assert!(
        shown["user_file"]["path"]
            .as_str()
            .unwrap()
            .ends_with(super::tests::USER_CONFIG_FILE)
    );
    assert!(
        shown["overrides"][0]
            .as_str()
            .unwrap()
            .starts_with("ui.dir=")
    );
}

#[test]
fn no_shipped_setting_is_hidden() {
    let fixture = Fixture::installed_with(None, Some(USER_FILE));
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert!(
        shown["settings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["hidden"] == false)
    );
}
