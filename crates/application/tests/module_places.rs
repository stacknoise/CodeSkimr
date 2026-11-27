//! The places behind a dependency between two modules (ADR 0108).

use codeskimr_application::error::CodedError;
use codeskimr_application::module_places::{Between, places_between};
use codeskimr_application::ports::{ReferencePlace, ReferencePlaces};
use codeskimr_domain::{
    ModelError, NamespaceGraph, NamespaceNode, Position, QualifiedName, RelativePath, SnapshotId,
    Span,
};

const SNAPSHOT: SnapshotId = SnapshotId::new(1);

struct Store {
    graph: NamespaceGraph,
    places: Vec<ReferencePlace>,
}

impl ReferencePlaces for Store {
    fn for_each_reference(
        &self,
        _snapshot: SnapshotId,
        visit: &mut dyn FnMut(ReferencePlace),
    ) -> Result<(), CodedError> {
        for place in &self.places {
            visit(place.clone());
        }
        Ok(())
    }

    fn namespaces(&self, _snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError> {
        Ok(self.graph.clone())
    }
}

fn place(path: &str, line: u32, ends: (&str, &str)) -> Result<ReferencePlace, ModelError> {
    let at = Position { line, column: 0 };
    Ok(ReferencePlace {
        path: RelativePath::new(path)?,
        span: Span::new(at, at)?,
        from: QualifiedName::new(ends.0)?,
        to: QualifiedName::new(ends.1)?,
    })
}

fn store() -> Result<Store, ModelError> {
    let mut graph = NamespaceGraph::default();
    for name in ["App.Web.Views", "App.Web.Api", "App.Domain.Model"] {
        let name = QualifiedName::new(name)?;
        graph.namespaces.push(NamespaceNode { name, types: 1 });
    }
    let places = vec![
        place(
            "web/views.rs",
            4,
            ("App.Web.Views.Board.draw", "App.Domain.Model.Item.new"),
        )?,
        place(
            "web/api.rs",
            9,
            ("App.Web.Api.Items.list", "App.Domain.Model.Item.name"),
        )?,
        place(
            "web/api.rs",
            12,
            ("App.Web.Api.Items.list", "App.Web.Views.Board.draw"),
        )?,
        place(
            "domain/item.rs",
            1,
            ("App.Domain.Model.Item.new", "App.Domain.Model.Item.name"),
        )?,
    ];
    Ok(Store { graph, places })
}

#[test]
fn the_places_of_an_edge_between_namespaces_are_listed_with_lines_from_one()
-> Result<(), ModelError> {
    let between = Between {
        from: "App.Web.Views",
        to: "App.Domain.Model",
        depth: 0,
    };
    let found =
        places_between(&store()?, SNAPSHOT, &between, 10).map_err(|_| ModelError::InvertedSpan)?;
    assert_eq!(found.total, 1);
    assert_eq!(found.places[0].path, "web/views.rs");
    assert_eq!(found.places[0].line, 5);
    assert_eq!(found.places[0].from, "App.Web.Views.Board.draw");
    Ok(())
}

#[test]
fn modules_merged_to_a_depth_add_up_the_places_of_their_namespaces() -> Result<(), ModelError> {
    let between = Between {
        from: "App.Web",
        to: "App.Domain",
        depth: 2,
    };
    let found =
        places_between(&store()?, SNAPSHOT, &between, 10).map_err(|_| ModelError::InvertedSpan)?;
    assert_eq!(found.total, 2);
    let lines: Vec<u32> = found.places.iter().map(|one| one.line).collect();
    assert_eq!(lines, [5, 10]);
    Ok(())
}

#[test]
fn a_limit_cuts_the_list_but_not_the_total() -> Result<(), ModelError> {
    let between = Between {
        from: "App.Web",
        to: "App.Domain",
        depth: 2,
    };
    let found =
        places_between(&store()?, SNAPSHOT, &between, 1).map_err(|_| ModelError::InvertedSpan)?;
    assert_eq!((found.total, found.places.len()), (2, 1));
    Ok(())
}

#[test]
fn the_other_direction_has_no_places_of_this_edge() -> Result<(), ModelError> {
    let between = Between {
        from: "App.Domain",
        to: "App.Web",
        depth: 2,
    };
    let found =
        places_between(&store()?, SNAPSHOT, &between, 10).map_err(|_| ModelError::InvertedSpan)?;
    assert_eq!(found.total, 0);
    Ok(())
}
