//! Storage for document bodies (Binary content, attachments, exports) that is
//! independent of the hosting cloud.
//!
//! Callers only use the [`DocumentStorage`] trait; each backend lives under
//! [`providers`] behind a cargo feature and is built with [`connect`].
//!
//! ```no_run
//! use haste_document_storage::{
//!     Document, DocumentKey, DocumentStorageKind, StorageError,
//!     providers::gcs::{GcsConfig, GcsCredentials},
//! };
//!
//! # async fn example() -> Result<(), StorageError> {
//! let storage = haste_document_storage::connect(DocumentStorageKind::Gcs(GcsConfig {
//!     bucket: "my-bucket".into(),
//!     prefix: Some("documents".into()),
//!     credentials: GcsCredentials::ApplicationDefault,
//!     endpoint: None,
//! }))
//! .await?;
//!
//! let key = DocumentKey::new("tenant/project/Binary/123")?;
//! storage
//!     .put(&key, Document::new(b"...".as_slice()).with_content_type("application/pdf"))
//!     .await?;
//! let document = storage.get(&key).await?;
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
    Document, DocumentBody, DocumentKey, DocumentMetadata, ListPage, ListRequest, StoredDocument,
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum DocumentStorageKind {
    #[cfg(feature = "filesystem")]
    Filesystem(providers::filesystem::FilesystemConfig),
    #[cfg(feature = "gcs")]
    Gcs(providers::gcs::GcsConfig),
}

/// Builds the configured backend and verifies that it can authenticate.
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
    }
}
