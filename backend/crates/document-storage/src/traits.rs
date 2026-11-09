use std::{future::Future, pin::Pin};

use crate::{
    error::StorageError,
    types::{
        Document, DocumentMetadata, DocumentName, DocumentScope, ListPage, ListRequest,
        StoredDocument,
    },
};

/// Boxed future returned by [`DocumentStorage`] methods, so the trait stays
/// object safe.
pub type StorageFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, StorageError>> + Send + 'a>>;

/// Object storage for document bodies. A document is addressed by the
/// [`DocumentScope`] (tenant and project) it belongs to and its
/// [`DocumentName`] within that scope.
///
/// An implementation wraps one bucket (or container, or pair of tables) and is
/// shared across tasks behind an `Arc`. How a scope and a name map onto the
/// backend, including any deployment-wide prefix, is the backend's concern.
pub trait DocumentStorage: Send + Sync {
    /// Stores `document` as `name` in `scope`, replacing whatever was there.
    fn put<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
        document: Document,
    ) -> StorageFuture<'a, DocumentMetadata>;

    /// Reads a document's metadata without its body. Fails with
    /// [`StorageError::NotFound`] when the scope holds no document of that
    /// name.
    fn head<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, DocumentMetadata>;

    /// Reads a document, streaming its body. Fails with
    /// [`StorageError::NotFound`] when the scope holds no document of that
    /// name.
    fn get<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, StoredDocument>;

    /// Deletes the document. Deleting a missing document succeeds.
    fn delete<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, ()>;

    /// Lists the documents of `scope` one page at a time, in name order. Pass
    /// the returned token back to fetch the next page.
    fn list<'a>(
        &'a self,
        scope: &'a DocumentScope,
        request: ListRequest,
    ) -> StorageFuture<'a, ListPage>;

    /// Every scope the store holds a document for. The one call that looks
    /// across tenants and projects, for maintenance such as the orphan sweep,
    /// which needs the scopes of projects that no longer exist.
    fn scopes(&self) -> StorageFuture<'_, Vec<DocumentScope>>;
}
