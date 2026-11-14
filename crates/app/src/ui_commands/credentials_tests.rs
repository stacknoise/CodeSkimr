//! The credential commands through the same path the server takes.

use codeskimr_application::ports::{AuthError, CredentialStore};
use codeskimr_codes::Coded;
use codeskimr_domain::CredentialId;
use secrecy::ExposeSecret;
use serde_json::{Value, json};

use super::tests::Fixture;

const SECRET: &str = "ghp_CANARY5d2a8e41b7c04f9a8e3d6c15";
/// A Git address that refuses at once: nothing listens on the discard port.
const GIT_ADDRESS: &str = "https://127.0.0.1:9/repo.git";

fn set(fixture: &Fixture, id: &str, secret: &str) -> Result<Value, &'static str> {
    fixture.call(
        "credentials.set",
        json!({ "id": id, "user": "anton", "secret": secret }),
    )
}

fn names(fixture: &Fixture) -> Value {
    fixture.call("credentials.list", Value::Null).unwrap()
}

#[test]
fn a_credential_is_stored_without_the_secret_in_the_answer() {
    let fixture = Fixture::installed(None);
    let answer = set(&fixture, "work", SECRET).unwrap();
    assert_eq!(answer, json!({ "id": "work" }));
    assert!(!answer.to_string().contains(SECRET));
    // The control: the keystore holds what was sent.
    let id = CredentialId::new("work").unwrap();
    let stored = fixture
        .secrets
        .with_credential(&id, |credential| {
            Ok(credential.secret().expose_secret().to_owned())
        })
        .unwrap();
    assert!(stored == SECRET, "the keystore holds another secret");
}

#[test]
fn a_credential_is_listed_once_a_project_names_it_and_removed_again() {
    let fixture = Fixture::installed(None);
    set(&fixture, "work", SECRET).unwrap();
    assert_eq!(names(&fixture), json!([]), "no project names it yet");
    // Registering stops at the clone: nothing listens at the address.
    let added = fixture.call(
        "projects.add",
        json!({ "source": GIT_ADDRESS, "credential": "work" }),
    );
    // Control: the add fails at the clone, but the project row exists and names the credential.
    assert!(added.is_err());
    assert_eq!(names(&fixture), json!([{ "id": "work", "stored": true }]));
    let removed = fixture.call("credentials.delete", json!({ "id": "work" }));
    assert_eq!(removed, Ok(json!({ "id": "work" })));
    assert_eq!(names(&fixture), json!([{ "id": "work", "stored": false }]));
    let id = CredentialId::new("work").unwrap();
    let kept = fixture.secrets.with_credential(&id, |_| Ok(()));
    assert!(matches!(kept, Err(AuthError::NotFound)));
}

#[test]
fn the_installation_token_and_bad_input_are_refused_without_echoing_the_secret() {
    let fixture = Fixture::installed(None);
    for (id, secret) in [
        ("installation-token", SECRET),
        ("Installation-Token", SECRET),
        (" installation-token", SECRET),
        ("work.refresh", SECRET),
        ("Work.Refresh ", SECRET),
        ("", SECRET),
        ("work", ""),
    ] {
        let result = set(&fixture, id, secret);
        assert_eq!(result, Err("API-101"), "{id}");
    }
    let no_user = fixture.call(
        "credentials.set",
        json!({ "id": "work", "user": " ", "secret": SECRET }),
    );
    assert_eq!(no_user, Err("API-101"));
    let wrong_type = fixture.call(
        "credentials.set",
        json!({ "id": "work", "user": "anton", "secret": [SECRET] }),
    );
    assert!(wrong_type.is_err());
    let id = CredentialId::new("work").unwrap();
    assert!(fixture.secrets.with_credential(&id, |_| Ok(())).is_err());
    let token = fixture.call("credentials.delete", json!({ "id": "installation-token" }));
    assert_eq!(token, Err("API-101"));
}

#[test]
fn a_project_cannot_log_in_with_the_installation_token_or_a_credential_for_a_folder() {
    let fixture = Fixture::installed(None);
    let token = fixture.call(
        "projects.add",
        json!({ "source": GIT_ADDRESS, "credential": "installation-token" }),
    );
    assert_eq!(token, Err("PRJ-101"));
    let folder = fixture.call(
        "projects.add",
        json!({ "source": fixture.folder(), "credential": "work" }),
    );
    assert_eq!(folder, Err("PRJ-101"));
}

#[test]
fn a_principal_without_the_rights_cannot_touch_credentials() {
    let fixture = Fixture::installed(None);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let nobody = crate::principal::nobody();
    let inputs = [
        (
            "credentials.set",
            json!({ "id": "work", "user": "anton", "secret": SECRET }),
        ),
        ("credentials.delete", json!({ "id": "work" })),
        ("credentials.list", Value::Null),
    ];
    for (id, input) in inputs {
        let refused = runtime
            .block_on(fixture.api.run(nobody, id, input))
            .map_err(|error| error.code().entry().id);
        assert_eq!(refused.unwrap_err(), "AUTH-401", "{id}");
    }
    // The control: nothing was stored, while the owner can store.
    let id = CredentialId::new("work").unwrap();
    assert!(fixture.secrets.with_credential(&id, |_| Ok(())).is_err());
    assert!(set(&fixture, "work", SECRET).is_ok());
}

/// A credential that was signed in with a refresh token, as `credential login` leaves it.
fn signed_in(fixture: &Fixture) {
    use codeskimr_application::ports::Credential;
    use codeskimr_secrets::{Provider, SignedIn};
    use secrecy::SecretString;

    use crate::refresh::{Link, save};

    let signed = SignedIn {
        credential: Credential::new(SecretString::from("oauth2"), SecretString::from("old")),
        refresh: Some(SecretString::from("old-refresh")),
    };
    let link = Link {
        provider: Provider::GitLab,
        client_id: "app",
        host: None,
    };
    let id = CredentialId::new("work").unwrap();
    save(&*fixture.secrets, &id, signed, &link).unwrap();
}

fn holds(fixture: &Fixture, name: &str) -> bool {
    let id = CredentialId::new(name).unwrap();
    fixture.secrets.with_credential(&id, |_| Ok(())).is_ok()
}

#[test]
fn the_refresh_entry_cannot_be_named_and_goes_with_its_credential() {
    let fixture = Fixture::installed(None);
    signed_in(&fixture);
    let delete_refresh = fixture.call("credentials.delete", json!({ "id": "Work.Refresh " }));
    assert_eq!(delete_refresh, Err("API-101"));
    assert!(holds(&fixture, "work.refresh"));
    // A new secret by hand must not be overwritten later by a token of the old sign-in.
    set(&fixture, "work", SECRET).unwrap();
    assert!(!holds(&fixture, "work.refresh"));
    signed_in(&fixture);
    fixture
        .call("credentials.delete", json!({ "id": "work" }))
        .unwrap();
    assert!(!holds(&fixture, "work") && !holds(&fixture, "work.refresh"));
}
