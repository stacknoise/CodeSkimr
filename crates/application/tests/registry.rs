//! Tests of the command registry and its central permission check.

use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{
    Command, CommandDescriptor, CommandId, Registry, RegistryError, Request,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, PermissionSet, Principal};

struct Double;
struct Shout;

const fn descriptor(
    id: &'static str,
    read_only: bool,
    permission: Permission,
) -> CommandDescriptor {
    CommandDescriptor {
        id: CommandId::new(id),
        title_key: "title",
        description_key: "description",
        shortcut: None,
        read_only,
        permission,
    }
}

impl Command for Double {
    type Input = u32;
    type Output = u32;
    const DESCRIPTOR: CommandDescriptor = descriptor("b.double", true, Permission::Read);

    fn execute(&self, request: Request<'_, u32>) -> Result<u32, CodedError> {
        Ok(request.input * 2)
    }
}

impl Command for Shout {
    type Input = String;
    type Output = String;
    const DESCRIPTOR: CommandDescriptor = descriptor("a.shout", false, Permission::Scan);

    fn execute(&self, request: Request<'_, String>) -> Result<String, CodedError> {
        Ok(request.input.to_uppercase())
    }
}

fn reader() -> Principal {
    Principal::limited(PermissionSet::none().with(Permission::Read))
}

#[test]
fn registered_commands_are_listed_in_id_order_and_executable_by_type() {
    let mut registry = Registry::new();
    registry.register(Double).unwrap();
    registry.register(Shout).unwrap();
    let ids: Vec<_> = registry.descriptors().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["a.shout", "b.double"]);
    let owner = Principal::local_owner();
    assert_eq!(registry.execute::<Double>(&owner, 4).unwrap(), 8);
    assert_eq!(registry.execute::<Shout>(&owner, "a".into()).unwrap(), "A");
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
    let error = registry
        .execute::<Shout>(&Principal::local_owner(), String::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnexpectedInternalError);
}

#[test]
fn a_principal_with_the_declared_permission_may_run_the_command() {
    let mut registry = Registry::new();
    registry.register(Double).unwrap();
    assert_eq!(registry.execute::<Double>(&reader(), 5).unwrap(), 10);
}

#[test]
fn a_principal_without_the_declared_permission_is_denied_before_the_command_runs() {
    let mut registry = Registry::new();
    registry.register(Shout).unwrap();
    let error = registry
        .execute::<Shout>(&reader(), "quiet".into())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PermissionDenied);
    let nobody = Principal::limited(PermissionSet::none());
    let error = registry
        .execute::<Shout>(&nobody, String::new())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PermissionDenied);
}

#[test]
fn a_denial_looks_the_same_for_every_missing_permission() {
    let mut registry = Registry::new();
    registry.register(Shout).unwrap();
    let nobody = Principal::limited(PermissionSet::none());
    let source_only = Principal::limited(PermissionSet::none().with(Permission::ReadSource));
    let first = registry
        .execute::<Shout>(&nobody, String::new())
        .unwrap_err();
    let second = registry
        .execute::<Shout>(&source_only, String::new())
        .unwrap_err();
    assert_eq!(first.to_string(), second.to_string());
}
