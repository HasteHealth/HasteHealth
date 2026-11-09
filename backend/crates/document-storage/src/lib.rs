//! Storage for document bodies (Binary content, attachments, exports) that is
//! independent of the hosting cloud.
//!
//! Callers only use the [`DocumentStorage`] trait; each backend lives under
//! [`providers`] behind a cargo feature and is built with [`connect`].
//!
//! Every call names the tenant and project the document belongs to
//! ([`DocumentScope`]) and the document's name within them ([`DocumentName`]),
//! so a caller can only ever reach its own project's documents.
//!
//! ```no_run
//! use haste_document_storage::{
//!     Document, DocumentName, DocumentScope, DocumentStorageKind,
//!     providers::gcs::{GcsConfig, GcsCredentials},
//! };
//! use haste_jwt::{ProjectId, TenantId};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = haste_document_storage::connect(DocumentStorageKind::Gcs(GcsConfig {
//!     bucket: "my-bucket".into(),
//!     prefix: Some("documents".into()),
//!     credentials: GcsCredentials::ApplicationDefault,
//!     endpoint: None,
//! }))
//! .await?;
//!
//! let scope = DocumentScope::new(
//!     &TenantId::new("acme".to_string())?,
//!     &ProjectId::new("clinic".to_string())?,
//! );
//! let name = DocumentName::new("Binary/123")?;
//! storage
//!     .put(
//!         &scope,
//!         &name,
//!         Document::new(b"...".as_slice()).with_content_type("application/pdf"),
//!     )
//!     .await?;
//! let document = storage.get(&scope, &name).await?;
//! let bytes = document.body.collect().await?;
//! # Ok(())
//! # }
//! ```

mod error;
mod traits;
mod types;

pub mod providers;
pub mod retry;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use error::StorageError;
pub use traits::{DocumentStorage, StorageFuture};
pub use types::{
    Document, DocumentBody, DocumentMetadata, DocumentName, DocumentScope, ListPage, ListRequest,
    StoredDocument,
};

/// The backend to connect to, with its configuration.
///
/// In configuration the backend is named by a `backend` field next to its
/// own settings:
///
/// ```toml
/// backend = "filesystem"
/// root = "/var/lib/haste-health/documents"
/// ```
///
/// ```toml
/// backend = "gcs"
/// bucket = "my-bucket"
/// prefix = "documents"
/// credentials = { type = "service_account_key_file", path = "/secrets/key.json" }
/// ```
///
/// ```toml
/// backend = "postgres"
/// database_url = "postgres://haste:password@localhost:5432/documents"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum DocumentStorageKind {
    #[cfg(feature = "filesystem")]
    Filesystem(providers::filesystem::FilesystemConfig),
    #[cfg(feature = "gcs")]
    Gcs(providers::gcs::GcsConfig),
    #[cfg(feature = "postgres")]
    Postgres(providers::postgres::PostgresConfig),
}

/// Builds the configured backend and verifies that it can authenticate. The
/// Postgres backend's tables must already exist (see
/// [`providers::postgres::PostgresDocumentStorage::migrate`]).
///
/// # Errors
///
/// Returns [`StorageError::Configuration`] or [`StorageError::Unauthorized`]
/// when the backend cannot be set up or its credentials are rejected.
pub async fn connect(kind: DocumentStorageKind) -> Result<Arc<dyn DocumentStorage>, StorageError> {
    match kind {
        #[cfg(feature = "filesystem")]
        DocumentStorageKind::Filesystem(config) => Ok(Arc::new(
            providers::filesystem::FilesystemDocumentStorage::connect(config).await?,
        )),
        #[cfg(feature = "gcs")]
        DocumentStorageKind::Gcs(config) => Ok(Arc::new(
            providers::gcs::GcsDocumentStorage::connect(config).await?,
        )),
        #[cfg(feature = "postgres")]
        DocumentStorageKind::Postgres(config) => Ok(Arc::new(
            providers::postgres::PostgresDocumentStorage::connect(config).await?,
        )),
    }
}
