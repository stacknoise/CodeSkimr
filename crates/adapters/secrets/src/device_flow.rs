//! OAuth device authorization grant (RFC 8628) for GitHub and GitLab (blueprint 12.2, point 2).
//!
//! The user approves the sign-in in a browser with a short code; the program never sees a password.
//! What comes back is a token with the scope the platform needs to read repositories. The device
//! code and the token are `SecretString`s, the answers of the server are wiped after parsing and
//! nothing of the flow is logged.

use std::io::Read;
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::Credential;
use codeskimr_codes::{Coded, ErrorCode};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use zeroize::Zeroizing;

use crate::consts::{
    ACCEPT_HEADER, ACCEPT_JSON, CONTENT_TYPE_FORM, DEFAULT_INTERVAL_SECS, ERROR_PENDING,
    ERROR_SLOW_DOWN, ESCAPE_MARK, ESCAPED_WIDTH, FIELD_CLIENT_ID, FIELD_DEVICE_CODE,
    FIELD_GRANT_TYPE, FIELD_REFRESH_TOKEN, FIELD_SCOPE, FORM_ASSIGN, FORM_JOIN, FORM_OVERHEAD,
    FORM_SAFE, GRANT_DEVICE_CODE, GRANT_REFRESH_TOKEN, HEX_RADIX, HTTPS_PREFIX, LOOPBACK_HOST,
    LOOPBACK_SCHEME, MAX_INTERVAL_SECS, MAX_LIFETIME_SECS, MAX_RESPONSE_BYTES, MIN_INTERVAL_SECS,
    NIBBLE_BITS, NIBBLE_MASK, PATH_SEPARATOR, PORT_SEPARATOR, REQUEST_TIMEOUT_SECS,
    SLOW_DOWN_STEP_SECS, USERINFO_MARKER,
};

/// Why the sign-in did not finish. The reason is not told: the answers of the platform may
/// contain codes that belong to the user.
#[derive(Debug, thiserror::Error)]
pub enum DeviceFlowError {
    /// The platform was not reachable, answered something else than the protocol, the user denied
    /// the sign-in, or the code expired.
    #[error("{}", ErrorCode::OauthFlowFailed.description())]
    Failed,
}

impl Coded for DeviceFlowError {
    fn code(&self) -> ErrorCode {
        ErrorCode::OauthFlowFailed
    }
}

impl From<DeviceFlowError> for CodedError {
    fn from(error: DeviceFlowError) -> Self {
        Self::caused_by(error)
    }
}

/// What the user has to do to approve the sign-in. Neither part is a secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    /// The page where the code is entered.
    pub verification_uri: String,
    /// The short code to enter there.
    pub user_code: String,
}

/// What a sign-in or a refresh brings back: the credential to log in with and, if the platform
/// issues one, the token that renews it. GitLab tokens live for two hours.
pub struct SignedIn {
    /// The access token with the user name of the platform.
    pub credential: Credential,
    /// The refresh token; platforms that rotate it send a new one with every refresh.
    pub refresh: Option<SecretString>,
}

/// One platform and one application, ready to sign in.
#[derive(Debug, Clone)]
pub struct DeviceFlow {
    device_url: String,
    token_url: String,
    client_id: String,
    scope: String,
    user: String,
}

#[derive(Deserialize)]
struct DeviceReply {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Deserialize)]
struct TokenReply {
    access_token: Option<SecretString>,
    refresh_token: Option<SecretString>,
    error: Option<String>,
}

impl DeviceReply {
    /// What the user sees. The texts are printed on a terminal, so control characters, which
    /// could rewrite its screen, fail the flow.
    fn prompt(&self) -> Result<Prompt, DeviceFlowError> {
        let clean = |text: &str| !text.chars().any(char::is_control);
        if clean(&self.verification_uri) && clean(&self.user_code) {
            Ok(Prompt {
                verification_uri: self.verification_uri.clone(),
                user_code: self.user_code.clone(),
            })
        } else {
            Err(DeviceFlowError::Failed)
        }
    }
}

impl DeviceFlow {
    /// A flow with the endpoints of the device code and of the token.
    ///
    /// # Errors
    /// `NET-601` if an endpoint is not an HTTPS address. Plain HTTP is only accepted for the
    /// loopback address, which the tests use.
    pub fn new(
        device_url: String,
        token_url: String,
        client_id: &str,
        scope: &str,
        user: &str,
    ) -> Result<Self, DeviceFlowError> {
        if !allowed(&device_url) || !allowed(&token_url) || client_id.is_empty() {
            return Err(DeviceFlowError::Failed);
        }
        Ok(Self {
            device_url,
            token_url,
            client_id: client_id.to_owned(),
            scope: scope.to_owned(),
            user: user.to_owned(),
        })
    }

    /// Asks for a code, shows it through `announce`, then polls until the user has approved.
    /// `wait` pauses between two polls; it is a parameter so that tests do not sleep.
    ///
    /// # Errors
    /// `NET-601` for every way the sign-in can fail.
    pub fn sign_in(
        &self,
        announce: impl FnOnce(&Prompt),
        wait: impl Fn(Duration),
    ) -> Result<Credential, DeviceFlowError> {
        self.sign_in_with_refresh(announce, wait)
            .map(|signed| signed.credential)
    }

    /// Like [`Self::sign_in`], and keeps the refresh token the platform sends along.
    ///
    /// # Errors
    /// `NET-601` for every way the sign-in can fail.
    pub fn sign_in_with_refresh(
        &self,
        announce: impl FnOnce(&Prompt),
        wait: impl Fn(Duration),
    ) -> Result<SignedIn, DeviceFlowError> {
        let client = client()?;
        let reply = self.request_code(&client)?;
        announce(&reply.prompt()?);
        let device_code = SecretString::from(reply.device_code);
        let lifetime = Duration::from_secs(reply.expires_in.min(MAX_LIFETIME_SECS));
        let mut interval = Duration::from_secs(
            reply
                .interval
                .unwrap_or(DEFAULT_INTERVAL_SECS)
                .clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS),
        );
        // The time is counted from the waits, so that the loop ends even if `wait` does not sleep.
        let mut waited = Duration::ZERO;
        while waited < lifetime {
            wait(interval);
            waited = waited.saturating_add(interval);
            let answer = self.poll(&client, &device_code)?;
            match (answer.access_token, answer.error.as_deref()) {
                (Some(token), _) if !token.expose_secret().is_empty() => {
                    return Ok(self.signed(token, answer.refresh_token));
                }
                (None, Some(ERROR_PENDING)) => {}
                (None, Some(ERROR_SLOW_DOWN)) => {
                    interval = interval
                        .saturating_add(Duration::from_secs(SLOW_DOWN_STEP_SECS))
                        .min(Duration::from_secs(MAX_INTERVAL_SECS));
                }
                _ => return Err(DeviceFlowError::Failed),
            }
        }
        Err(DeviceFlowError::Failed)
    }

    /// Trades a refresh token for a new access token (RFC 6749, section 6). The application
    /// is a public client, so no client secret is sent.
    ///
    /// # Errors
    /// `NET-601` if the platform is not reachable or refuses the token (it was revoked or has
    /// been used already).
    pub fn refresh(&self, refresh_token: &SecretString) -> Result<SignedIn, DeviceFlowError> {
        let body = form(&[
            (FIELD_CLIENT_ID, self.client_id.as_str()),
            (FIELD_REFRESH_TOKEN, refresh_token.expose_secret()),
            (FIELD_GRANT_TYPE, GRANT_REFRESH_TOKEN),
        ]);
        let answer: TokenReply = post(&client()?, &self.token_url, &body)?;
        match answer.access_token {
            Some(token) if !token.expose_secret().is_empty() => {
                Ok(self.signed(token, answer.refresh_token))
            }
            _ => Err(DeviceFlowError::Failed),
        }
    }

    fn signed(&self, token: SecretString, refresh: Option<SecretString>) -> SignedIn {
        SignedIn {
            credential: Credential::new(SecretString::from(self.user.clone()), token),
            refresh: refresh.filter(|text| !text.expose_secret().is_empty()),
        }
    }

    fn request_code(&self, client: &Client) -> Result<DeviceReply, DeviceFlowError> {
        let body = form(&[
            (FIELD_CLIENT_ID, self.client_id.as_str()),
            (FIELD_SCOPE, self.scope.as_str()),
        ]);
        post(client, &self.device_url, &body)
    }

    fn poll(
        &self,
        client: &Client,
        device_code: &SecretString,
    ) -> Result<TokenReply, DeviceFlowError> {
        let body = form(&[
            (FIELD_CLIENT_ID, self.client_id.as_str()),
            (FIELD_DEVICE_CODE, device_code.expose_secret()),
            (FIELD_GRANT_TYPE, GRANT_DEVICE_CODE),
        ]);
        post(client, &self.token_url, &body)
    }
}

/// Whether the endpoint may be used: HTTPS, or plain HTTP to the loopback address itself. The
/// host is compared exactly after the scheme: `127.0.0.1.example.com` and `127.0.0.1@host` are
/// other hosts. A URL with user info is refused.
fn allowed(url: &str) -> bool {
    let (secure, rest) = match (
        url.strip_prefix(HTTPS_PREFIX),
        url.strip_prefix(LOOPBACK_SCHEME),
    ) {
        (Some(rest), _) => (true, rest),
        (None, Some(rest)) => (false, rest),
        (None, None) => return false,
    };
    let authority = rest.split(PATH_SEPARATOR).next().unwrap_or_default();
    if authority.contains(USERINFO_MARKER) {
        return false;
    }
    let host = authority.split(PORT_SEPARATOR).next().unwrap_or_default();
    secure && !host.is_empty() || host == LOOPBACK_HOST
}

fn client() -> Result<Client, DeviceFlowError> {
    Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        // A redirect would carry the device code to another host.
        .redirect(Policy::none())
        .build()
        .map_err(|_| DeviceFlowError::Failed)
}

/// Sends the form and reads the JSON answer. The text of the answer is wiped when it is parsed.
/// `reqwest` keeps its own copy of the body until the request is done (blueprint 15, library
/// copies).
fn post<T: for<'de> Deserialize<'de>>(
    client: &Client,
    url: &str,
    body: &Zeroizing<String>,
) -> Result<T, DeviceFlowError> {
    let response = client
        .post(url)
        .header(ACCEPT_HEADER, ACCEPT_JSON)
        .header(reqwest::header::CONTENT_TYPE, CONTENT_TYPE_FORM)
        .body(body.as_str().to_owned())
        .send()
        .map_err(|_| DeviceFlowError::Failed)?;
    let mut text = Zeroizing::new(String::new());
    response
        .take(MAX_RESPONSE_BYTES)
        .read_to_string(&mut text)
        .map_err(|_| DeviceFlowError::Failed)?;
    serde_json::from_str(&text).map_err(|_| DeviceFlowError::Failed)
}

/// `application/x-www-form-urlencoded` body of `fields`. The buffer is wiped when dropped,
/// because one of the values is the device code.
fn form(fields: &[(&str, &str)]) -> Zeroizing<String> {
    // The room for the worst case (every byte escaped) is reserved at once: a buffer that grows
    // leaves unwiped copies of the values behind.
    let room = fields
        .iter()
        .map(|(name, value)| (name.len() + value.len()) * ESCAPED_WIDTH + FORM_OVERHEAD)
        .sum();
    let mut out = Zeroizing::new(String::with_capacity(room));
    for (index, (name, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push(FORM_JOIN);
        }
        escape_into(&mut out, name);
        out.push(FORM_ASSIGN);
        escape_into(&mut out, value);
    }
    out
}

fn escape_into(out: &mut String, text: &str) {
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || FORM_SAFE.contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push(ESCAPE_MARK);
            for nibble in [byte >> NIBBLE_BITS, byte & NIBBLE_MASK] {
                let digit = char::from_digit(u32::from(nibble), HEX_RADIX).unwrap_or_default();
                out.push(digit.to_ascii_uppercase());
            }
        }
    }
}
