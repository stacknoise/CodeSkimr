//! Who acts and what they may do (ADR 0019, "Seam in 1.0").
//!
//! Every use case receives a [`Principal`]. In 1.0 there is exactly one kind of principal, the
//! [`Principal::local_owner`] of the single-user installation, who holds every permission. The
//! types are pure: no login, no storage, no I/O. A team edition adds principals produced by an
//! authenticator without changing the signatures that carry them.

use crate::consts::{
    PERMISSION_ADMIN, PERMISSION_MANAGE, PERMISSION_READ, PERMISSION_READ_SOURCE,
    PERMISSION_REGISTER_PROJECT, PERMISSION_SCAN,
};
use crate::vocabulary::labeled_enum;

labeled_enum!(
    /// One thing a principal may be allowed to do.
    Permission {
        /// Read structure, metrics and search results.
        Read => PERMISSION_READ,
        /// Read source text; every route, tool and export that returns source needs this.
        ReadSource => PERMISSION_READ_SOURCE,
        /// Start scans.
        Scan => PERMISSION_SCAN,
        /// Manage a project and its members.
        Manage => PERMISSION_MANAGE,
        /// Register new projects.
        RegisterProject => PERMISSION_REGISTER_PROJECT,
        /// Administer the installation.
        Admin => PERMISSION_ADMIN,
    }
);

impl Permission {
    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A set of permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PermissionSet(u8);

impl PermissionSet {
    /// The set that allows nothing.
    #[must_use]
    pub const fn none() -> Self {
        Self(0)
    }

    /// The set that holds every permission.
    #[must_use]
    pub const fn all() -> Self {
        Self::none()
            .with(Permission::Read)
            .with(Permission::ReadSource)
            .with(Permission::Scan)
            .with(Permission::Manage)
            .with(Permission::RegisterProject)
            .with(Permission::Admin)
    }

    /// This set plus `permission`.
    #[must_use]
    pub const fn with(self, permission: Permission) -> Self {
        Self(self.0 | permission.bit())
    }

    /// Whether `permission` is in the set.
    #[must_use]
    pub const fn contains(self, permission: Permission) -> bool {
        self.0 & permission.bit() != 0
    }
}

impl FromIterator<Permission> for PermissionSet {
    fn from_iter<I: IntoIterator<Item = Permission>>(permissions: I) -> Self {
        permissions.into_iter().fold(Self::none(), Self::with)
    }
}

/// The authenticated caller of a use case.
///
/// [`Principal::local_owner`] is for the single-user composition root only; a principal of the
/// team edition comes from the authenticator through [`Principal::limited`]. Adapters never
/// create a principal themselves, they receive the one the composition root resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Principal {
    permissions: PermissionSet,
}

impl Principal {
    /// The owner of a single-user installation: no login, every permission.
    #[must_use]
    pub const fn local_owner() -> Self {
        Self {
            permissions: PermissionSet::all(),
        }
    }

    /// A principal that holds exactly `permissions` (team edition, restricted tokens, tests).
    #[must_use]
    pub const fn limited(permissions: PermissionSet) -> Self {
        Self { permissions }
    }

    /// Whether this principal holds `permission`.
    #[must_use]
    pub const fn holds(&self, permission: Permission) -> bool {
        self.permissions.contains(permission)
    }
}
