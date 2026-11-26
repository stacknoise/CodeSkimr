//! The architecture rules (ADR 0104 to 0107): evaluation against stored references and the held
//! state.

use codeskimr_application::config::{Config, ConfigLayer, LayerSource, Rules};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ReferencePlace, ReferencePlaces};
use codeskimr_application::rules::{Baseline, RuleKind, evaluate};
use codeskimr_domain::Position;
use codeskimr_domain::{QualifiedName, RelativePath, SnapshotId, Span};

const SNAPSHOT: SnapshotId = SnapshotId::new(1);

struct Fake(Vec<ReferencePlace>);

impl ReferencePlaces for Fake {
    fn for_each_reference(
        &self,
        _snapshot: SnapshotId,
        visit: &mut dyn FnMut(ReferencePlace),
    ) -> Result<(), CodedError> {
        self.0.iter().cloned().for_each(visit);
        Ok(())
    }
}

fn place(path: &str, line: u32, from: &str, to: &str) -> ReferencePlace {
    let at = Position { line, column: 4 };
    ReferencePlace {
        path: RelativePath::new(path).unwrap_or_else(|e| unreachable!("{e:?}")),
        span: Span::new(at, at).unwrap_or_else(|e| unreachable!("{e:?}")),
        from: QualifiedName::new(from).unwrap_or_else(|e| unreachable!("{e:?}")),
        to: QualifiedName::new(to).unwrap_or_else(|e| unreachable!("{e:?}")),
    }
}

fn rules(text: &str) -> Rules {
    let layer =
        ConfigLayer::parse(LayerSource::Project, text).unwrap_or_else(|e| unreachable!("{e:?}"));
    Config::resolve(&[layer])
        .unwrap_or_else(|e| unreachable!("{e:?}"))
        .rules
}

const PROJECT: &str = "[[rules.forbid]]\nfrom = \"Shop.Reports\"\nto = \"Shop.Checkout\"\n\n[[rules.public_api]]\nmodule = \"Shop.Data\"\ninternal = [\"Shop.Data.Internal\"]\n";

fn references() -> Fake {
    Fake(vec![
        place(
            "Reports/A.cs",
            3,
            "Shop.Reports.A.Run",
            "Shop.Checkout.Cart.Total",
        ),
        place(
            "Reports/A.cs",
            9,
            "Shop.Reports.A.Run",
            "Shop.Checkout.Cart.Total",
        ),
        place(
            "Reports/A.cs",
            12,
            "Shop.Reports.A.Run",
            "Shop.Data.OrderRepository.Get",
        ),
        place(
            "Api/C.cs",
            5,
            "Shop.Api.C.Get",
            "Shop.Data.Internal.Cache.Read",
        ),
        place(
            "Data/R.cs",
            7,
            "Shop.Data.OrderRepository.Get",
            "Shop.Data.Internal.Cache.Read",
        ),
        place(
            "Reports/B.cs",
            1,
            "Shop.ReportsX.B.Run",
            "Shop.Checkout.Cart.Total",
        ),
    ])
}

#[test]
fn a_forbidden_use_is_one_violation_with_every_place() {
    let outcomes = evaluate(&rules(PROJECT), &references(), SNAPSHOT).unwrap();
    let forbid = &outcomes[0];
    assert_eq!(forbid.id, "forbid:Shop.Reports>Shop.Checkout");
    assert_eq!(forbid.kind, RuleKind::Forbid);
    let [violation] = forbid.violations.as_slice() else {
        panic!("one violation, found {}", forbid.violations.len());
    };
    assert_eq!(violation.from.as_str(), "Shop.Reports.A.Run");
    let lines: Vec<u32> = violation
        .places
        .iter()
        .map(|p| p.span.start().line)
        .collect();
    assert_eq!(lines, [3, 9]);
}

#[test]
fn the_inside_of_a_module_is_closed_to_others_but_open_to_the_module_itself() {
    let outcomes = evaluate(&rules(PROJECT), &references(), SNAPSHOT).unwrap();
    let api = &outcomes[1];
    assert_eq!(api.id, "public_api:Shop.Data");
    let found: Vec<_> = api.violations.iter().map(|v| v.from.as_str()).collect();
    assert_eq!(found, ["Shop.Api.C.Get"]);
}

#[test]
fn a_rule_without_violation_is_in_the_answer_with_an_empty_list() {
    let outcomes = evaluate(&rules(PROJECT), &Fake(Vec::new()), SNAPSHOT).unwrap();
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(|outcome| outcome.violations.is_empty()));
}

#[test]
fn holding_the_state_makes_the_violations_old_and_new_ones_stay_new() {
    let held = evaluate(&rules(PROJECT), &references(), SNAPSHOT).unwrap();
    let baseline = Baseline::holding("2026-11-26", &held);
    assert_eq!(baseline.entries.len(), 2);
    let text = baseline.to_text().unwrap();
    let baseline = Baseline::parse(&text).unwrap();

    let mut more = references().0;
    more.push(place(
        "Reports/A.cs",
        40,
        "Shop.Reports.A.Other",
        "Shop.Checkout.Cart.Pay",
    ));
    let judged = baseline.judge(evaluate(&rules(PROJECT), &Fake(more), SNAPSHOT).unwrap());
    let forbid = &judged.outcomes[0];
    assert_eq!(forbid.violations.len(), 2);
    assert_eq!(forbid.new_violations(), 1);
    assert!(judged.resolved.is_empty());
}

#[test]
fn a_held_violation_that_is_gone_is_listed_as_resolved() {
    let held = evaluate(&rules(PROJECT), &references(), SNAPSHOT).unwrap();
    let baseline = Baseline::holding("2026-11-26", &held);
    let judged = baseline.judge(evaluate(&rules(PROJECT), &Fake(Vec::new()), SNAPSHOT).unwrap());
    assert_eq!(judged.resolved.len(), 2);
}

#[test]
fn a_broken_held_state_is_refused() {
    assert!(Baseline::parse("held_on = [").is_err());
    assert!(Baseline::parse("held_on = \"x\"\nunknown = 1\n").is_err());
}
