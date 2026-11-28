//! The settings overview through the same path the server takes.

use serde_json::{Value, json};

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

fn save(fixture: &Fixture, key: &str, value: &Value) -> Result<Value, &'static str> {
    fixture.call("ui_settings.set_value", json!({"key": key, "value": value}))
}

#[test]
fn a_saved_number_shows_up_as_the_value_of_the_next_scan() {
    let fixture = Fixture::installed_with(None, Some(USER_FILE));
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(setting(&shown, "scan.max_workers")["edit"], "scan");
    assert_eq!(setting(&shown, "scan.max_workers")["default"], "0");
    save(&fixture, "scan.max_workers", &json!(3)).unwrap();
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(setting(&shown, "scan.max_workers")["value"], "3");
    assert_eq!(setting(&shown, "scan.max_workers")["origin"], "ui");
    save(&fixture, "scan.max_workers", &json!(null)).unwrap();
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(setting(&shown, "scan.max_workers")["value"], "2");
}

#[test]
fn values_read_at_start_keep_showing_the_running_value_until_a_restart() {
    let fixture = Fixture::installed_with(None, Some(USER_FILE));
    let before = fixture.call("settings.effective", Value::Null).unwrap();
    let started = setting(&before, "watcher.debounce_ms");
    assert_eq!(started["edit"], "start");
    assert!(started["pending"].is_null());
    save(&fixture, "watcher.debounce_ms", &json!(500)).unwrap();
    let after = fixture.call("settings.effective", Value::Null).unwrap();
    let shown = setting(&after, "watcher.debounce_ms");
    assert_eq!(shown["value"], started["value"]);
    assert_eq!(shown["pending"], "500");
}

#[test]
fn only_numbers_and_switches_of_the_allowed_tables_can_be_saved() {
    let fixture = Fixture::new();
    for (key, value) in [
        ("level2.enabled", json!(true)),
        ("server.port", json!(80)),
        ("ui.dir", json!(1)),
        ("made.up", json!(1)),
        ("scan.max_workers", json!("many")),
        ("scan.max_workers", json!(1.5)),
    ] {
        assert_eq!(save(&fixture, key, &value), Err("API-101"), "{key}");
    }
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(setting(&shown, "level2.enabled")["edit"], "locked");
    assert_eq!(setting(&shown, "ui.dir")["edit"], "cli");
}

#[test]
fn a_value_that_makes_the_configuration_invalid_is_not_saved() {
    let fixture = Fixture::new();
    assert_eq!(
        save(&fixture, "similarity.ngram_size", &json!(0)),
        Err("CFG-101")
    );
    let shown = fixture.call("settings.effective", Value::Null).unwrap();
    assert_eq!(
        setting(&shown, "similarity.ngram_size")["origin"],
        "default"
    );
}

#[test]
fn only_a_manager_may_save_a_setting() {
    let fixture = Fixture::new();
    let answer = fixture.call_as(
        crate::principal::reader(),
        "ui_settings.set_value",
        json!({"key": "scan.max_workers", "value": 2}),
    );
    assert_eq!(answer.unwrap_err(), "AUTH-401");
}
