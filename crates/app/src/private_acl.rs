//! Access rules for the folder of the launch files on Windows (issue #104).
//!
//! Unix gives the folder mode 0700. Windows has no such mode: a folder gets the rules of its
//! parent, and below a drive root (a data folder on `D:\`) these usually let every signed-in user
//! read it. So the rules inherited from the parent are dropped and only the account of the user is
//! granted access. Files created in the folder inherit that rule.

use std::ffi::OsString;
use std::path::Path;

#[cfg(windows)]
use crate::consts::ACL_TOOL;
use crate::consts::{
    ACCOUNT_SEPARATOR, ACL_FULL_RULE, ACL_GRANT, ACL_NO_INHERIT, ACL_QUIET, ENV_USERDOMAIN,
    ENV_USERNAME,
};

/// The account of the user as `DOMAIN\name`, or just the name without a domain. `None` if the
/// environment does not say who the user is.
pub(crate) fn account(env: impl Fn(&str) -> Option<String>) -> Option<String> {
    let name = env(ENV_USERNAME).filter(|name| !name.is_empty())?;
    Some(
        match env(ENV_USERDOMAIN).filter(|domain| !domain.is_empty()) {
            Some(domain) => [domain, name].join(ACCOUNT_SEPARATOR),
            None => name,
        },
    )
}

/// The arguments of the access tool: no inherited rules, full access for `account` only.
pub(crate) fn arguments(dir: &Path, account: &str) -> Vec<OsString> {
    let mut rule = OsString::from(account);
    rule.push(ACL_FULL_RULE);
    vec![
        dir.as_os_str().to_owned(),
        OsString::from(ACL_NO_INHERIT),
        OsString::from(ACL_GRANT),
        rule,
        OsString::from(ACL_QUIET),
    ]
}

/// Restricts `dir` to the user. Fails if the user is unknown or the tool does not succeed: a
/// folder whose rules are not known is not used for a launch code.
#[cfg(windows)]
pub(crate) fn restrict_to_user(dir: &Path) -> std::io::Result<()> {
    use std::process::{Command, Stdio};

    let account = account(|name| std::env::var(name).ok())
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
    let status = Command::new(ACL_TOOL)
        .args(arguments(dir, &account))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::ErrorKind::PermissionDenied.into())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        let map: HashMap<_, _> = pairs.iter().copied().collect();
        move |name| map.get(name).map(|value| (*value).to_owned())
    }

    #[test]
    fn the_account_has_a_domain_when_there_is_one() {
        let both = env(&[("USERNAME", "anton"), ("USERDOMAIN", "WORKGROUP")]);
        assert_eq!(account(both).as_deref(), Some("WORKGROUP\\anton"));
        let name_only = env(&[("USERNAME", "anton")]);
        assert_eq!(account(name_only).as_deref(), Some("anton"));
        let nobody = env(&[("USERDOMAIN", "WORKGROUP")]);
        assert_eq!(account(nobody), None);
    }

    #[test]
    fn only_the_user_gets_access_and_nothing_is_inherited() {
        let args = arguments(Path::new("C:\\Users\\a\\launch"), "PC\\a");
        let text: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();
        assert_eq!(
            text,
            [
                "C:\\Users\\a\\launch",
                "/inheritance:r",
                "/grant:r",
                "PC\\a:(OI)(CI)F",
                "/q"
            ]
        );
    }
}
