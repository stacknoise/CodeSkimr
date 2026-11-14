//! The project list: projects with what their newest snapshot says.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{ProjectCatalog, SnapshotStore};
use codeskimr_domain::{ScanMode, SymbolKind};
use common::{data, file, header, project, scanned, store, symbol};

#[test]
fn a_project_without_a_scan_has_no_snapshot_and_no_languages() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let overviews = store.overviews().unwrap();
    assert_eq!(overviews.len(), 1);
    assert_eq!(overviews[0].latest, None);
    assert_eq!(overviews[0].types, 0);
    assert_eq!(overviews[0].languages, Vec::new());
}

#[test]
fn types_and_languages_come_from_the_newest_snapshot() {
    let store = scanned(
        &[],
        vec![
            symbol(1, 11, SymbolKind::Class, "App.Order", None),
            symbol(2, 11, SymbolKind::Method, "App.Order.Total", Some(1)),
        ],
        Vec::new(),
        Vec::new(),
    );
    let overviews = store.overviews().unwrap();
    let overview = &overviews[0];
    assert_eq!(overview.types, 1);
    assert_eq!(overview.latest.as_ref().unwrap().id.get(), 10);
    assert_eq!(overview.languages.len(), 1);
    assert_eq!(overview.languages[0].language.as_str(), "csharp");
    assert_eq!(overview.languages[0].loc, 10);
    assert_eq!(overview.languages[0].symbols, 2);
}

#[test]
fn projects_come_in_the_order_they_were_registered() {
    let store = store(3);
    store.save_project(&project(1, "/work/a", &[])).unwrap();
    store.save_project(&project(2, "/work/b", &[])).unwrap();
    let names: Vec<_> = store
        .overviews()
        .unwrap()
        .into_iter()
        .map(|overview| overview.project.id.get())
        .collect();
    assert_eq!(names, vec![1, 2]);
    let mut scan = data(header(10, ScanMode::Full));
    scan.files = vec![file(10, 11, "a.cs", "h")];
    store.persist(scan).unwrap();
    assert!(store.overviews().unwrap()[0].latest.is_some());
}
