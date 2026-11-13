//! Tests of the command registry.

use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{
    Command, CommandDescriptor, CommandId, Registry, RegistryError,
};

struct Double;
struct Shout;

const fn descriptor(id: &'static str, read_only: bool) -> CommandDescriptor {
    CommandDescriptor {
        id: CommandId::new(id),
        title_key: "title",
        description_key: "description",
        shortcut: None,
        read_only,
    }
}

impl Command for Double {
    type Input = u32;
    type Output = u32;
    const DESCRIPTOR: CommandDescriptor = descriptor("b.double", true);

    fn execute(&self, input: u32) -> Result<u32, CodedError> {
        Ok(input * 2)
    }
}

impl Command for Shout {
    type Input = String;
    type Output = String;
    const DESCRIPTOR: CommandDescriptor = descriptor("a.shout", false);

    fn execute(&self, input: String) -> Result<String, CodedError> {
        Ok(input.to_uppercase())
    }
}

#[test]
fn registered_commands_are_listed_in_id_order_and_executable_by_type() {
    let mut registry = Registry::new();
    registry.register(Double).unwrap();
    registry.register(Shout).unwrap();
    let ids: Vec<_> = registry.descriptors().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["a.shout", "b.double"]);
    assert_eq!(registry.handler::<Double>().unwrap().execute(4).unwrap(), 8);
    assert!(
        !registry
            .descriptor(CommandId::new("a.shout"))
            .unwrap()
            .read_only
    );
    assert!(registry.descriptor(CommandId::new("none")).is_none());
}

#[test]
fn duplicate_id_is_rejected_and_changes_nothing() {
    let mut registry = Registry::new();
    registry.register(Double).unwrap();
    assert_eq!(
        registry.register(Double),
        Err(RegistryError::DuplicateId(Double::DESCRIPTOR.id))
    );
    assert_eq!(registry.descriptors().count(), 1);
    assert!(registry.handler::<Shout>().is_none());
}
