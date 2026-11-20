//! `codeskimr annotations` and `codeskimr alerts` over a real, scanned project: the text and the
//! JSON they print, and the confirmation that `dismiss-all` needs.

use std::fs;
use std::path::PathBuf;

use codeskimr_cli::{
    AddAnnotation, AlertsAction, AnnotationFields, AnnotationsAction, ById, DismissAlerts,
    GlobalOptions, ListAlerts, ListAnnotations, PageArgs, ScanArgs, SymbolArg,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::ScanMode;
use codeskimr_i18n::Messages;
use serde_json::Value;

use crate::annotations_cli::{Surroundings, run_alerts, run_annotations};
use crate::locations::Locations;
use crate::mcp_http_tests::locations;
use crate::memory_keystore::secrets;
use crate::scan::{self, Session};

const TARGET: &str = "namespace Demo;\n\npublic class Target\n{\n    public static int Do(int x)\n    {\n        return x + 1;\n    }\n}\n";
const PROJECT: &str = "1";

struct World {
    _root: tempfile::TempDir,
    folder: PathBuf,
    data: Locations,
    session: Session,
    global: GlobalOptions,
    messages: Messages,
}

impl World {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("project");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("Target.cs"), TARGET).unwrap();
        let data = locations(&root.path().join("data"));
        let args = ScanArgs {
            source: folder.to_string_lossy().into_owned(),
            branch: None,
            credential: None,
            json: false,
            watch: false,
        };
        let global = GlobalOptions::default();
        let session =
            scan::open(scan::prepare(&args).unwrap(), &global, &data, &secrets()).unwrap();
        session.scan_quick(ScanMode::Full).unwrap();
        Self {
            _root: root,
            folder,
            data,
            session,
            global,
            messages: Messages::load().unwrap(),
        }
    }

    fn around<'a>(&'a self, language: &'a str) -> Surroundings<'a> {
        Surroundings {
            global: &self.global,
            locations: &self.data,
            messages: &self.messages,
            language,
        }
    }

    fn edit_target(&self) {
        fs::write(
            self.folder.join("Target.cs"),
            TARGET.replace("x + 1", "x + 2"),
        )
        .unwrap();
        self.session.scan_quick(ScanMode::Incremental).unwrap();
    }
}

fn method() -> SymbolArg {
    SymbolArg {
        kind: "method".to_owned(),
        name: "Demo.Target.Do".to_owned(),
        parameters: Vec::new(),
    }
}

fn add(kind: &str, text: &str, json: bool) -> AnnotationsAction {
    AnnotationsAction::Add(AddAnnotation {
        project: PROJECT.to_owned(),
        symbol: method(),
        kind: kind.to_owned(),
        fields: AnnotationFields {
            text: Some(text.to_owned()),
            ..AnnotationFields::default()
        },
        json,
    })
}

fn list(json: bool) -> AnnotationsAction {
    AnnotationsAction::List(ListAnnotations {
        project: PROJECT.to_owned(),
        symbol: None,
        kind: None,
        status: None,
        page: PageArgs::default(),
        json,
    })
}

fn alert_list(json: bool) -> AlertsAction {
    AlertsAction::List(ListAlerts {
        project: PROJECT.to_owned(),
        state: None,
        kinds: Vec::new(),
        page: PageArgs::default(),
        json,
    })
}

fn dismiss_all(confirm: bool, json: bool) -> AlertsAction {
    AlertsAction::DismissAll(DismissAlerts {
        project: PROJECT.to_owned(),
        kinds: Vec::new(),
        confirm,
        json,
    })
}

fn json_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn an_annotation_is_added_listed_in_both_forms_and_removed() {
    let world = World::new();
    let en = world.around("en");
    let saved = run_annotations(&add("critical", "Billing relies on it", false), &en).unwrap();
    assert!(saved.contains("Demo.Target.Do"), "{saved}");
    let text = run_annotations(&list(false), &en).unwrap();
    assert!(text.contains("Billing relies on it"), "{text}");
    let german = run_annotations(&list(false), &world.around("de")).unwrap();
    assert!(german.contains("Billing relies on it"), "{german}");
    let listed = json_of(&run_annotations(&list(true), &en).unwrap());
    assert_eq!(listed["total"], 1);
    assert_eq!(listed["items"][0]["kind"], "critical");
    let id = listed["items"][0]["id"].as_u64().unwrap();
    let removed = AnnotationsAction::Remove(ById {
        project: PROJECT.to_owned(),
        id,
        json: true,
    });
    let answer = json_of(&run_annotations(&removed, &en).unwrap());
    assert_eq!(answer["id"], id);
    assert_eq!(
        json_of(&run_annotations(&list(true), &en).unwrap())["total"],
        0
    );
}

#[test]
fn control_characters_of_a_note_do_not_reach_the_terminal_but_json_keeps_them() {
    let world = World::new();
    let en = world.around("en");
    let nasty = "ok\u{1b}]0;title\u{7}\u{1b}[31mred\u{9b}x\ttab";
    run_annotations(&add("note", nasty, false), &en).unwrap();
    let text = run_annotations(&list(false), &en).unwrap();
    assert!(
        !text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t'),
        "{text:?}"
    );
    assert!(text.contains("ok"), "{text:?}");
    let json = run_annotations(&list(true), &en).unwrap();
    assert!(!json.contains('\u{1b}'), "JSON escapes it: {json}");
    assert!(json.contains("\\u001b"), "{json}");
    assert_eq!(
        crate::annotations_cli::printable("a\tb\nc\u{85}d"),
        "a\tb\ncd"
    );
}

#[test]
fn an_unknown_project_or_symbol_is_a_coded_failure() {
    let world = World::new();
    let en = world.around("en");
    let mut unknown_project = list(false);
    if let AnnotationsAction::List(args) = &mut unknown_project {
        "999".clone_into(&mut args.project);
    }
    let error = run_annotations(&unknown_project, &en).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ProjectNotFound);
    let mut missing = add("note", "text", false);
    if let AnnotationsAction::Add(args) = &mut missing {
        "No.Such.Symbol".clone_into(&mut args.symbol.name);
    }
    let error = run_annotations(&missing, &en).unwrap_err();
    assert_eq!(error.code(), ErrorCode::AnnotationTargetNotFound);
}

#[test]
fn dismissing_everything_needs_the_explicit_yes() {
    let world = World::new();
    let en = world.around("en");
    run_annotations(&add("critical", "Billing relies on it", false), &en).unwrap();
    world.edit_target();
    let open = json_of(&run_alerts(&alert_list(true), &en).unwrap());
    assert_eq!(open["open"], 1);
    assert_eq!(open["items"][0]["kind"], "changed");
    let refused = run_alerts(&dismiss_all(false, false), &en).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::RequestInvalid);
    assert_eq!(
        json_of(&run_alerts(&alert_list(true), &en).unwrap())["open"],
        1,
        "nothing was dismissed"
    );
    let done = json_of(&run_alerts(&dismiss_all(true, true), &en).unwrap());
    assert_eq!(done["dismissed"], 1);
    assert_eq!(done["open"], 0);
    let text = run_alerts(&alert_list(false), &en).unwrap();
    assert!(
        text.contains("No alerts") || text.contains("dismissed"),
        "{text}"
    );
}
