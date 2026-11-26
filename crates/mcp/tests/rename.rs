//! `preview_rename` against fake ports.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, PermissionSet, Principal};
use codeskimr_mcp::RenameArgs;
use common::tools;

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn args(new_name: &str) -> RenameArgs {
    RenameArgs {
        symbol: "1".to_owned(),
        new_name: new_name.to_owned(),
        project: None,
        limit: None,
    }
}

#[test]
fn the_places_that_have_to_change_are_counted_by_file_and_by_how_sure_they_are() {
    let answer = tools().preview_rename(&owner(), &args("Purchase")).unwrap();
    assert_eq!(answer["symbol"]["qualified_name"], "App.Domain.Order");
    assert_eq!(answer["old_name"], "Order");
    assert_eq!(answer["new_name"], "Purchase");
    let sites = &answer["sites"];
    assert_eq!(sites["total"], 2);
    assert_eq!(sites["files"], 2);
    assert_eq!(sites["verified"], 2);
    assert_eq!(sites["unchecked"], 0);
    assert_eq!(answer["preliminary"], false);
    assert_eq!(answer["public_interface"], true);
    assert_eq!(answer["snapshot"]["id"], 10);
}

#[test]
fn a_place_names_its_file_position_container_and_layer() {
    let answer = tools().preview_rename(&owner(), &args("Purchase")).unwrap();
    let items = &answer["sites"]["items"];
    let declaration = &items[1];
    assert_eq!(declaration["kind"], "declaration");
    assert_eq!(declaration["path"], "src/order.cs");
    assert_eq!(declaration["line"], 5);
    assert_eq!(declaration["layer"], "Domain");
    let usage = &items[0];
    assert_eq!(usage["kind"], "usage");
    assert_eq!(usage["path"], "src/app/service.cs");
    assert_eq!(
        (usage["line"].as_u64(), usage["column"].as_u64()),
        (Some(12), Some(8))
    );
    assert_eq!(
        usage["from"]["qualified_name"],
        "App.Application.Service.Run"
    );
    assert_eq!(usage["layer"], "Application");
}

#[test]
fn text_hits_leave_out_the_lines_a_place_already_explains() {
    let answer = tools().preview_rename(&owner(), &args("Purchase")).unwrap();
    let hits = &answer["text_hits"];
    assert_eq!(hits["searched"], true);
    assert_eq!(hits["total"], 2);
    let kinds: Vec<_> = hits["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|hit| (hit["path"].as_str().unwrap(), hit["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("config/app.json", "configuration"),
            ("docs/api.md", "documentation")
        ]
    );
    assert_eq!(hits["items"][1]["line"], 120);
}

#[test]
fn a_caller_who_may_not_read_source_gets_no_text_hits() {
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    let answer = tools().preview_rename(&reader, &args("Purchase")).unwrap();
    assert_eq!(answer["text_hits"]["searched"], false);
    assert_eq!(answer["text_hits"]["total"], 0);
    assert_eq!(answer["sites"]["total"], 2);
}

#[test]
fn a_symbol_that_already_has_the_new_name_in_its_scope_is_a_conflict() {
    let answer = tools().preview_rename(&owner(), &args("Invoice")).unwrap();
    let conflicts = answer["conflicts"].as_array().unwrap();
    assert_eq!(conflicts.len(), 1, "{conflicts:?}");
    assert_eq!(conflicts[0]["qualified_name"], "App.Domain.Invoice");
    assert_eq!(conflicts[0]["path"], "src/order.cs");
    // `Order()` and `Invoice()` differ only in the name: they clash.
    assert_eq!(conflicts[0]["same_signature"], true);
    let none = tools().preview_rename(&owner(), &args("Purchase")).unwrap();
    assert!(none["conflicts"].as_array().unwrap().is_empty());
}

#[test]
fn the_list_is_cut_to_the_limit_but_the_counts_stay_whole() {
    let one = RenameArgs {
        limit: Some(1),
        ..args("Purchase")
    };
    let answer = tools().preview_rename(&owner(), &one).unwrap();
    assert_eq!(answer["sites"]["total"], 2);
    assert_eq!(answer["sites"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(answer["text_hits"]["total"], 2);
    assert_eq!(answer["text_hits"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(answer["notice"]["code"], "MCP-501");
}

#[test]
fn a_name_that_is_not_a_name_or_is_the_old_one_is_an_invalid_argument() {
    for wrong in ["", "  ", "two words", "9lives", "Order", "a-b"] {
        let error = tools().preview_rename(&owner(), &args(wrong)).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid, "{wrong:?}");
    }
}

#[test]
fn an_unknown_symbol_and_a_caller_without_read_are_refused() {
    let unknown = RenameArgs {
        symbol: "999".to_owned(),
        ..args("Purchase")
    };
    let error = tools().preview_rename(&owner(), &unknown).unwrap_err();
    assert_eq!(error.code(), ErrorCode::SymbolNotFound);
    let nobody = Principal::limited(PermissionSet::none());
    let error = tools()
        .preview_rename(&nobody, &args("Purchase"))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PermissionDenied);
}
