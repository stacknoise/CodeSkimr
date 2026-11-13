//! Which layer owns a qualified name.

#[cfg(test)]
mod tests {
    use codeskimr_domain::{Layer, LayerName, QualifiedName};

    fn layer() -> Layer {
        Layer {
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
}
