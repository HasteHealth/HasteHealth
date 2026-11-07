//! Repository configuration and construction.
//!
//! Lives here rather than in each binary so the server and the worker connect
//! to the same store described the same way — a second copy of these types is
//! a second thing to keep in step, and the two drifting apart is how a worker
//! ends up indexing into somewhere the server never reads.

use std::sync::Arc;

use haste_document_storage::{
    DocumentStorageKind,
    providers::{
        filesystem::FilesystemConfig,
        gcs::GcsConfig,
        postgres::{PostgresConfig, PostgresDocumentStorage},
    },
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

use crate::{binary_storage::BinaryStorage, pg::PGConnection};

/// Where the FHIR server stores its resources.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum RepoConfig {
    Postgres(PostgresRepoConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresRepoConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for PostgresRepoConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@127.0.0.1/haste_health".to_string(),
            max_connections: 20,
        }
    }
}

impl Default for RepoConfig {
    fn default() -> Self {
        RepoConfig::Postgres(PostgresRepoConfig::default())
    }
}

/// Document storage for resource content kept out of the database. Today
/// that is `Binary.data` of at least `binary_min_bytes` bytes.
///
/// The backend is named by a `backend` field, next to its own settings. The
/// default is `postgres`: tables in the repository database, so a deployment
/// needs nothing beyond the database it already has. `none` keeps all content
/// in the resource rows.
///
/// ```toml
/// [document_storage]
/// backend = "filesystem"
/// root = "/var/lib/haste-health/documents"
/// binary_min_bytes = 65536
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentStorageConfig {
    /// Smallest `Binary.data`, in bytes after base64 decoding, that is kept in
    /// document storage. Smaller content stays in the database.
    #[serde(default = "default_binary_min_bytes")]
    pub binary_min_bytes: usize,
    /// How often the async-job worker removes orphaned documents, in seconds.
    /// `0` turns the automatic sweep off.
    #[serde(default = "default_sweep_seconds")]
    pub sweep_interval_seconds: u64,
    /// How old an orphaned document must be before the sweep removes it, in
    /// seconds. It must comfortably exceed the longest transaction.
    #[serde(default = "default_sweep_seconds")]
    pub sweep_grace_seconds: u64,
    #[serde(flatten)]
    pub backend: DocumentBackend,
}

/// The document storage backend, or none.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum DocumentBackend {
    /// All content stays in the database.
    None,
    /// Tables in a PostgreSQL database: the repository database unless
    /// `database_url` names another.
    Postgres(PostgresConfig),
    /// A directory on a local or mounted filesystem.
    Filesystem(FilesystemConfig),
    /// A Google Cloud Storage bucket.
    Gcs(GcsConfig),
}

impl Default for DocumentStorageConfig {
    fn default() -> Self {
        Self {
            binary_min_bytes: default_binary_min_bytes(),
            sweep_interval_seconds: default_sweep_seconds(),
            sweep_grace_seconds: default_sweep_seconds(),
            backend: DocumentBackend::Postgres(PostgresConfig::default()),
        }
    }
}

fn default_binary_min_bytes() -> usize {
    64 * 1024
}

fn default_sweep_seconds() -> u64 {
    24 * 60 * 60
}

/// The backend `config` selects, with a Postgres backend's unset database
/// filled in from `repo`. `None` when document storage is off.
fn storage_kind(config: &DocumentStorageConfig, repo: &RepoConfig) -> Option<DocumentStorageKind> {
    match &config.backend {
        DocumentBackend::None => None,
        DocumentBackend::Postgres(postgres) => {
            let mut postgres = postgres.clone();
            if postgres.database_url.is_none() {
                let RepoConfig::Postgres(repo) = repo;
                postgres.database_url = Some(repo.database_url.clone());
            }
            Some(DocumentStorageKind::Postgres(postgres))
        }
        DocumentBackend::Filesystem(filesystem) => {
            Some(DocumentStorageKind::Filesystem(filesystem.clone()))
        }
        DocumentBackend::Gcs(gcs) => Some(DocumentStorageKind::Gcs(gcs.clone())),
    }
}

/// Connects the configured document storage; `None` when it is off.
///
/// # Errors
///
/// Returns an error when the backend cannot be set up or rejects its
/// credentials.
pub async fn connect_binary_storage(
    config: &DocumentStorageConfig,
    repo: &RepoConfig,
) -> Result<Option<BinaryStorage>, OperationOutcomeError> {
    let Some(kind) = storage_kind(config, repo) else {
        return Ok(None);
    };
    let storage = haste_document_storage::connect(kind).await?;
    Ok(Some(BinaryStorage::new(storage, config.binary_min_bytes)))
}

/// Creates or updates the Postgres backend's tables. Other backends need no
/// migration.
///
/// # Errors
///
/// Returns an error when the database cannot be reached or a migration fails.
pub async fn migrate_document_storage(
    config: &DocumentStorageConfig,
    repo: &RepoConfig,
) -> Result<(), OperationOutcomeError> {
    if let Some(DocumentStorageKind::Postgres(postgres)) = storage_kind(config, repo) {
        PostgresDocumentStorage::connect(postgres)
            .await?
            .migrate()
            .await?;
    }
    Ok(())
}

/// Opens a connection pool for `config`.
///
/// # Errors
///
/// Returns an error if the pool cannot open its first connection — an
/// unreachable host, bad credentials, or a malformed `database_url`.
pub async fn create_repo(config: &RepoConfig) -> Result<Arc<PGConnection>, OperationOutcomeError> {
    match config {
        RepoConfig::Postgres(pg_config) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(pg_config.max_connections)
                .connect(&pg_config.database_url)
                .await
                .map_err(|e| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        format!("Failed to connect to the repository database: {e}"),
                    )
                })?;

            Ok(Arc::new(PGConnection::pool(pool)))
        }
    }
}
