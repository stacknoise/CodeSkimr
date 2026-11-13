//! Tests of principals and permissions.

use std::str::FromStr;

use codeskimr_domain::{Permission, PermissionSet, Principal};

#[test]
fn the_local_owner_holds_every_permission() {
    let owner = Principal::local_owner();
    assert!(Permission::ALL.iter().all(|p| owner.holds(*p)));
}

#[test]
fn a_limited_principal_holds_only_what_it_was_given() {
    let principal = Principal::limited(PermissionSet::from_iter([
        Permission::Read,
        Permission::Scan,
    ]));
    assert!(principal.holds(Permission::Read));
    assert!(principal.holds(Permission::Scan));
    for denied in [
        Permission::ReadSource,
        Permission::Manage,
        Permission::RegisterProject,
        Permission::Admin,
    ] {
        assert!(!principal.holds(denied));
    }
    assert!(
        Permission::ALL
            .iter()
            .all(|p| !Principal::limited(PermissionSet::none()).holds(*p))
    );
}

#[test]
fn permission_labels_round_trip() {
    for permission in Permission::ALL {
        assert_eq!(Permission::from_str(permission.label()), Ok(*permission));
    }
    assert!(Permission::from_str("root").is_err());
}
