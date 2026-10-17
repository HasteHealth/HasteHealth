//! `OAuth2` access tokens for the Google Cloud Storage API.

use std::{
    fmt::{self, Debug},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::http::{check, parse_json, transport};
use crate::{error::StorageError, retry::with_retry};

const SCOPE: &str = "https://www.googleapis.com/auth/devstorage.read_write";
const DEFAULT_TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const DEFAULT_METADATA_HOST: &str = "metadata.google.internal";
const METADATA_TOKEN_PATH: &str = "/computeMetadata/v1/instance/service-accounts/default/token";
/// Tokens are refreshed this long before they expire.
const REFRESH_MARGIN: Duration = Duration::from_secs(60);
/// Lifetime requested for service account assertions, the longest Google allows.
const ASSERTION_LIFETIME: Duration = Duration::from_secs(3600);

/// Where the Google credentials come from.
#[derive(Clone)]
pub enum GcsCredentials {
    /// Google's Application Default Credentials lookup: the file named by
    /// `GOOGLE_APPLICATION_CREDENTIALS`, then the file written by
    /// `gcloud auth application-default login`, then the metadata server.
    ApplicationDefault,
    /// A service account key file as downloaded from the Cloud Console.
    ServiceAccountKeyFile(PathBuf),
    /// The contents of a service account key file.
    ServiceAccountKeyJson(String),
    /// The metadata server of GCE, GKE, Cloud Run and similar runtimes.
    MetadataServer,
    /// A fixed `OAuth2` access token, for tests and emulators.
    AccessToken(String),
    /// No `Authorization` header at all, for emulators such as fake-gcs-server.
    Anonymous,
}

/// Redacts key material and tokens.
impl Debug for GcsCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApplicationDefault => f.write_str("ApplicationDefault"),
            Self::ServiceAccountKeyFile(path) => {
                f.debug_tuple("ServiceAccountKeyFile").field(path).finish()
            }
            Self::ServiceAccountKeyJson(_) => f.write_str("ServiceAccountKeyJson(REDACTED)"),
            Self::MetadataServer => f.write_str("MetadataServer"),
            Self::AccessToken(_) => f.write_str("AccessToken(REDACTED)"),
            Self::Anonymous => f.write_str("Anonymous"),
        }
    }
}

/// Hands out bearer tokens, refreshing short-lived ones before they expire.
pub(super) struct Authenticator {
    source: TokenSource,
    cached: Mutex<Option<CachedToken>>,
}

enum TokenSource {
    /// Always the same value; `None` sends no `Authorization` header.
    Static(Option<String>),
    Refreshing(Refresh),
}

/// A way to obtain a fresh access token.
enum Refresh {
    ServiceAccount {
        email: String,
        key: EncodingKey,
        token_uri: String,
    },
    AuthorizedUser(AuthorizedUser),
    MetadataServer {
        url: String,
    },
}

struct CachedToken {
    value: String,
    expires_at: Instant,
}

impl Authenticator {
    pub(super) async fn new(credentials: &GcsCredentials) -> Result<Self, StorageError> {
        let source = match credentials {
            GcsCredentials::ApplicationDefault => application_default().await?,
            GcsCredentials::ServiceAccountKeyFile(path) => from_file(path).await?,
            GcsCredentials::ServiceAccountKeyJson(json) => from_json(json)?,
            GcsCredentials::MetadataServer => TokenSource::Refreshing(Refresh::MetadataServer {
                url: metadata_url(),
            }),
            GcsCredentials::AccessToken(token) => TokenSource::Static(Some(token.clone())),
            GcsCredentials::Anonymous => TokenSource::Static(None),
        };

        Ok(Self {
            source,
            cached: Mutex::new(None),
        })
    }

    /// The bearer token to send, or `None` for anonymous access.
    pub(super) async fn bearer(&self, client: &Client) -> Result<Option<String>, StorageError> {
        let refresh = match &self.source {
            TokenSource::Static(token) => return Ok(token.clone()),
            TokenSource::Refreshing(refresh) => refresh,
        };

        // Holding the lock while fetching means one refresh at a time.
        let mut cached = self.cached.lock().await;
        if let Some(token) = cached.as_ref()
            && token.expires_at.saturating_duration_since(Instant::now()) > REFRESH_MARGIN
        {
            return Ok(Some(token.value.clone()));
        }

        let token = with_retry("token", || refresh.fetch(client))
            .await
            .map_err(credentials_rejected)?;
        let value = token.value.clone();
        *cached = Some(token);
        Ok(Some(value))
    }
}

impl Refresh {
    async fn fetch(&self, client: &Client) -> Result<CachedToken, StorageError> {
        let requested_at = Instant::now();
        let request = match self {
            Self::ServiceAccount {
                email,
                key,
                token_uri,
            } => client.post(token_uri).form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &sign_assertion(email, key, token_uri)?),
            ]),
            Self::AuthorizedUser(user) => client.post(DEFAULT_TOKEN_URI).form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &user.client_id),
                ("client_secret", &user.client_secret),
                ("refresh_token", &user.refresh_token),
            ]),
            Self::MetadataServer { url } => client.get(url).header("Metadata-Flavor", "Google"),
        };

        let response = request.send().await.map_err(transport)?;
        let token: TokenResponse = parse_json(check(response, "token endpoint").await?).await?;
        Ok(CachedToken {
            value: token.access_token,
            expires_at: requested_at + Duration::from_secs(token.expires_in),
        })
    }
}

/// Token endpoints answer bad credentials with 400 as often as with 401.
fn credentials_rejected(error: StorageError) -> StorageError {
    match error {
        StorageError::Backend(400 | 401 | 403, message) | StorageError::Forbidden(message) => {
            StorageError::Unauthorized(message)
        }
        other => other,
    }
}

#[derive(Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    #[serde(default = "default_token_uri")]
    token_uri: String,
}

fn default_token_uri() -> String {
    DEFAULT_TOKEN_URI.to_string()
}

#[derive(Deserialize)]
struct AuthorizedUser {
    client_id: String,
    client_secret: String,
    refresh_token: String,
}

#[derive(Serialize)]
struct AssertionClaims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

/// A signed JWT that the token endpoint exchanges for an access token.
fn sign_assertion(email: &str, key: &EncodingKey, token_uri: &str) -> Result<String, StorageError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StorageError::Configuration(format!("system clock is invalid: {error}")))?
        .as_secs();
    let claims = AssertionClaims {
        iss: email,
        scope: SCOPE,
        aud: token_uri,
        iat: now,
        exp: now + ASSERTION_LIFETIME.as_secs(),
    };
    jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, key).map_err(|error| {
        StorageError::Configuration(format!("could not sign service account assertion: {error}"))
    })
}

async fn application_default() -> Result<TokenSource, StorageError> {
    if let Some(path) = std::env::var_os("GOOGLE_APPLICATION_CREDENTIALS") {
        return from_file(Path::new(&path)).await;
    }
    if let Some(path) = gcloud_credentials_file()
        && tokio::fs::try_exists(&path).await.unwrap_or(false)
    {
        return from_file(&path).await;
    }
    Ok(TokenSource::Refreshing(Refresh::MetadataServer {
        url: metadata_url(),
    }))
}

/// Where `gcloud auth application-default login` writes its credentials.
fn gcloud_credentials_file() -> Option<PathBuf> {
    let config_dir = std::env::var_os("CLOUDSDK_CONFIG")
        .map(PathBuf::from)
        .or_else(|| {
            if cfg!(windows) {
                std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join("gcloud"))
            } else {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/gcloud"))
            }
        })?;
    Some(config_dir.join("application_default_credentials.json"))
}

fn metadata_url() -> String {
    let host =
        std::env::var("GCE_METADATA_HOST").unwrap_or_else(|_| DEFAULT_METADATA_HOST.to_string());
    format!("http://{host}{METADATA_TOKEN_PATH}")
}

async fn from_file(path: &Path) -> Result<TokenSource, StorageError> {
    let json = tokio::fs::read_to_string(path).await.map_err(|error| {
        StorageError::Configuration(format!(
            "could not read credentials file {}: {error}",
            path.display()
        ))
    })?;
    from_json(&json)
}

/// Parses a credentials file: a service account key or an authorized user
/// (the shape `gcloud auth application-default login` produces).
fn from_json(json: &str) -> Result<TokenSource, StorageError> {
    let invalid = |error: serde_json::Error| {
        StorageError::Configuration(format!("invalid Google credentials JSON: {error}"))
    };
    let value: serde_json::Value = serde_json::from_str(json).map_err(invalid)?;
    let kind = value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);

    let refresh = match kind.as_deref() {
        Some("service_account") => {
            let account: ServiceAccountKey = serde_json::from_value(value).map_err(invalid)?;
            let key =
                EncodingKey::from_rsa_pem(account.private_key.as_bytes()).map_err(|error| {
                    StorageError::Configuration(format!(
                        "invalid service account private key: {error}"
                    ))
                })?;
            Refresh::ServiceAccount {
                email: account.client_email,
                key,
                token_uri: account.token_uri,
            }
        }
        Some("authorized_user") => {
            Refresh::AuthorizedUser(serde_json::from_value(value).map_err(invalid)?)
        }
        other => {
            return Err(StorageError::Configuration(format!(
                "unsupported Google credentials type {other:?}; expected service_account or authorized_user"
            )));
        }
    };
    Ok(TokenSource::Refreshing(refresh))
}
