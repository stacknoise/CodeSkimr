//! Which layer owns a qualified name.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{Layer, LayerName, QualifiedName};

    fn layer() -> Layer {
        Layer {
            may_use: None,
            name: LayerName::new("Domain").unwrap(),
            namespaces: vec![QualifiedName::new("App.Domain").unwrap()],
        }
    }

    #[test]
    fn owns_the_namespace_and_everything_below_it() {
        assert!(layer().owns("App.Domain"));
        assert!(layer().owns("App.Domain.Order.Total"));
    }

    #[test]
    fn does_not_own_a_name_that_only_shares_the_prefix_text() {
        assert!(!layer().owns("App.DomainX.Order"));
        assert!(!layer().owns("App"));
    }

    #[test]
    fn owns_what_lies_below_it_in_module_paths_too() {
        let layer = Layer {
            may_use: None,
            name: LayerName::new("Domain").unwrap(),
            namespaces: vec![QualifiedName::new("crates::domain").unwrap()],
        };
        assert!(layer.owns("crates::domain::src::lib::Order"));
        assert!(!layer.owns("crates::domain_extra::src::lib"));
    }

    #[test]
    fn a_layer_named_by_namespace_owns_the_declarations_of_a_namespaced_language() {
        let layer = Layer {
            may_use: None,
            name: LayerName::new("Demo").unwrap(),
            namespaces: vec![QualifiedName::new("Demo").unwrap()],
        };
        assert!(layer.owns("Demo.Calculator.Add"));
        assert!(!layer.owns("app/Calculator.cs"));
    }

    #[test]
    fn allows_every_layer_without_a_limit_and_only_the_listed_ones_with_one() {
        let name = |text: &str| LayerName::new(text).unwrap();
        let mut layer = layer();
        assert!(layer.allows(&name("Infra")));
        layer.may_use = Some(vec![name("Core")]);
        assert!(layer.allows(&name("Core")));
        assert!(!layer.allows(&name("Infra")));
        assert!(layer.allows(&layer.name.clone()));
        layer.may_use = Some(Vec::new());
        assert!(!layer.allows(&name("Core")));
    }
}
