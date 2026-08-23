//! In-memory CLI state for a single invocation, plus the on-disk locations it's seeded from.

use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use haste_server::auth_n::oidc::routes::discovery::WellKnownDiscoveryDocument;
use tokio::sync::Mutex;

use crate::cli::{config::CliConfiguration, secrets::CliSecrets};

/// Directory holding the CLI's config and secrets files (`~/.haste_health`).
static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = std::env::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".haste_health");

    std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    config_dir
});

/// Non-secret profile config (server URLs, auth mode). Safe to inspect or back up.
pub(crate) static CONFIG_LOCATION: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_DIR.join("config.toml"));

/// Client secrets and cached OAuth tokens, kept out of `CONFIG_LOCATION` so they never
/// show up via `config show-profile` or similar.
pub(crate) static SECRETS_LOCATION: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_DIR.join(".secrets.toml"));

pub(crate) struct CliState {
    pub(crate) config: CliConfiguration,
    pub(crate) secrets: CliSecrets,
    pub(crate) access_token: Option<String>,
    pub(crate) well_known_document: Option<WellKnownDiscoveryDocument>,
}

impl CliState {
    fn new(config: CliConfiguration, secrets: CliSecrets) -> Self {
        CliState {
            config,
            secrets,
            access_token: None,
            well_known_document: None,
        }
    }
}

/// Lazily loads `CONFIG_LOCATION`/`SECRETS_LOCATION` from disk on first access.
pub(crate) static CLI_STATE: LazyLock<Arc<Mutex<CliState>>> = LazyLock::new(|| {
    let config = crate::cli::config::load_config(&CONFIG_LOCATION);
    let secrets = crate::cli::secrets::load_secrets(&SECRETS_LOCATION);

    Arc::new(Mutex::new(CliState::new(config, secrets)))
});
