use std::{future::Future, pin::Pin};

use crate::{
    error::StorageError,
    types::{Document, DocumentKey, DocumentMetadata, ListPage, ListRequest, StoredDocument},
};

/// Boxed future returned by [`DocumentStorage`] methods, so the trait stays
/// object safe.
pub type StorageFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, StorageError>> + Send + 'a>>;

/// Object storage for document bodies, keyed by [`DocumentKey`].
///
/// An implementation wraps one bucket (or container) and is shared across
/// tasks behind an `Arc`. Keys are opaque to callers: how they map onto the
/// backend, including any deployment-wide prefix, is the backend's concern.
pub trait DocumentStorage: Send + Sync {
    /// Stores `document` under `key`, replacing whatever was there.
    fn put<'a>(
        &'a self,
        key: &'a DocumentKey,
        document: Document,
    ) -> StorageFuture<'a, DocumentMetadata>;

    /// Reads a document's metadata without its body.
    /// Fails with [`StorageError::NotFound`] when nothing is stored under `key`.
    fn head<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, DocumentMetadata>;

    /// Reads a document, streaming its body.
    /// Fails with [`StorageError::NotFound`] when nothing is stored under `key`.
    fn get<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, StoredDocument>;

    /// Deletes the document under `key`. Deleting a missing document succeeds.
    fn delete<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, ()>;

    /// Lists documents whose keys start with the request's prefix, one page at
    /// a time; pass the returned token back to fetch the next page.
    fn list(&self, request: ListRequest) -> StorageFuture<'_, ListPage>;
}
