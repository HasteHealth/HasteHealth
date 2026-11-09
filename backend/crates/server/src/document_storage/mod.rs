//! Large `Binary.data` kept in document storage instead of the resource rows.
//!
//! The `[document_storage]` section ([`DocumentStorageConfig`]) selects the
//! storage and the size from which content goes there. The clinical lane's
//! [`BinaryContent`] layer moves content of at least `binary_min_bytes` to the
//! storage on the way in and puts it back on the way out, so the repository
//! only ever stores a row-sized Binary and a client gets the same Binary
//! either way. The row keeps `data` with only the marker extension
//! ([`MARKER_URL`]), whose value is the document's name.
//!
//! Every storage call names the tenant and project of the request
//! ([`DocumentScope`](haste_document_storage::DocumentScope)), and a name only means something within that scope, so
//! a marker can never reach another project's document, whatever it says.
//!
//! Code that reads the repository directly rather than through the FHIR
//! client sees the row form. The search indexer wants exactly that; nothing
//! else reads `Binary.data` that way.
//!
//! A document is written before its row, so a write that fails or rolls back
//! afterwards leaves a document no version refers to; [`sweep`] removes those.
//!
//! [`BinaryContent`]: crate::fhir_client::middleware::binary_content::BinaryContent

mod sweep;

use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use haste_document_storage::{
    DocumentName, DocumentStorage, DocumentStorageKind, StorageError,
    providers::{
        filesystem::FilesystemConfig,
        gcs::GcsConfig,
        postgres::{PostgresConfig, PostgresDocumentStorage},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::{config::RepoConfig, utilities::generate_id};
use serde::{Deserialize, Serialize};

use crate::config::ServerConfig;

pub use sweep::{SweepReport, sweep};

/// Marks a `Binary.data` whose value is in document storage; the extension's
/// `valueString` is the document's name within the project's
/// [`DocumentScope`](haste_document_storage::DocumentScope). Only the server sets it: one sent by a client is removed
/// before the Binary is stored.
pub const MARKER_URL: &str = "https://haste.health/document-storage";

/// Document storage for resource content kept out of the database. Today
/// that is `Binary.data` of at least `binary_min_bytes` bytes.
///
/// The backend is named by a `backend` field, next to its own settings. The
/// default is `postgres`: a table in the repository database, so a deployment
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
    /// A table in a PostgreSQL database: the repository database unless
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

/// The document storage Binary content goes to, and the size from which it
/// does.
#[derive(Clone)]
pub struct BinaryStorage {
    storage: Arc<dyn DocumentStorage>,
    min_bytes: usize,
}

impl BinaryStorage {
    /// Content of at least `min_bytes` bytes (after base64 decoding) is kept
    /// in `storage`.
    #[must_use]
    pub fn new(storage: Arc<dyn DocumentStorage>, min_bytes: usize) -> Self {
        Self { storage, min_bytes }
    }

    #[must_use]
    pub fn storage(&self) -> &Arc<dyn DocumentStorage> {
        &self.storage
    }

    #[must_use]
    pub fn min_bytes(&self) -> usize {
        self.min_bytes
    }
}

impl Debug for BinaryStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryStorage")
            .field("min_bytes", &self.min_bytes)
            .finish_non_exhaustive()
    }
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

/// Connects the document storage `config` selects; `None` when it is off.
///
/// # Errors
///
/// Returns an error when the backend cannot be set up or rejects its
/// credentials.
pub async fn connect(
    config: &ServerConfig,
) -> Result<Option<BinaryStorage>, OperationOutcomeError> {
    let Some(kind) = storage_kind(&config.document_storage, &config.repo) else {
        return Ok(None);
    };
    let storage = haste_document_storage::connect(kind).await?;
    Ok(Some(BinaryStorage::new(
        storage,
        config.document_storage.binary_min_bytes,
    )))
}

/// Creates or updates the Postgres backend's table. Other backends need no
/// migration.
///
/// # Errors
///
/// Returns an error when the database cannot be reached or a migration fails.
pub async fn migrate(config: &ServerConfig) -> Result<(), OperationOutcomeError> {
    if let Some(DocumentStorageKind::Postgres(postgres)) =
        storage_kind(&config.document_storage, &config.repo)
    {
        PostgresDocumentStorage::connect(postgres)
            .await?
            .migrate()
            .await?;
    }
    Ok(())
}

/// The name of a new Binary document, `Binary/{document id}`, within the
/// project's [`DocumentScope`](haste_document_storage::DocumentScope). The id is generated here, so a name exists
/// before the resource has its own id and version.
///
/// # Errors
///
/// Fails when the generated name is not a valid [`DocumentName`], which it
/// always is.
pub fn new_document_name() -> Result<DocumentName, StorageError> {
    DocumentName::new(format!("Binary/{}", generate_id(None)))
}
