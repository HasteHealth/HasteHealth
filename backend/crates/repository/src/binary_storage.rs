//! `Binary.data` kept in document storage instead of the database.
//!
//! When document storage is configured, a Binary written with at least
//! `min_bytes` of content has the content put in storage under
//! `{tenant}/{project}/Binary/{id}/{versionId}`, and the database row keeps
//! `data` with only an extension ([`STORED_CONTENT_EXTENSION_URL`]) saying the
//! value is there. Reads put the value back, so a caller gets the same Binary
//! either way. Smaller content, and all content when storage is not
//! configured, stays in the row.
//!
//! Every version has its own document, the same way every version has its own
//! row. A document is only written before its row, so a document whose row
//! never committed (a rolled back transaction, a failed insert) is an orphan
//! that [`crate::pg::PGConnection::sweep_binary_documents`] removes.

use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_util::{StreamExt as _, TryStreamExt as _, stream};
use haste_document_storage::{Document, DocumentKey, DocumentStorage, StorageError};
use haste_fhir_model::r4::generated::{
    resources::{Binary, Resource},
    terminology::IssueType,
    types::{Extension, ExtensionValueTypeChoice, FHIRBoolean, Meta},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};

/// Marks a `Binary.data` whose value is in document storage. Only the server
/// sets it: one sent by a client is removed before the Binary is stored.
pub const STORED_CONTENT_EXTENSION_URL: &str = "https://haste.health/document-storage";

/// Documents fetched at once when a read returns several stored Binaries.
const FETCH_CONCURRENCY: usize = 8;

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
}

impl Debug for BinaryStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryStorage")
            .field("min_bytes", &self.min_bytes)
            .finish_non_exhaustive()
    }
}

/// The key a Binary version's content is stored under.
///
/// # Errors
///
/// Fails when the parts do not make a valid [`DocumentKey`].
pub fn document_key(
    tenant: &TenantId,
    project: &ProjectId,
    id: &str,
    version_id: &str,
) -> Result<DocumentKey, StorageError> {
    DocumentKey::new(format!(
        "{}/{}/Binary/{id}/{version_id}",
        tenant.as_ref(),
        project.as_ref()
    ))
}

/// The tenant, project and version id a key made by [`document_key`] names,
/// or `None` for any other key.
#[must_use]
pub fn parse_document_key(key: &str) -> Option<(&str, &str, &str)> {
    let mut parts = key.split('/');
    let (Some(tenant), Some(project), Some("Binary"), Some(_id), Some(version_id), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return None;
    };
    Some((tenant, project, version_id))
}

/// Content taken off a Binary for its database row, to be put back on the
/// copy handed to the caller.
pub(crate) struct Offloaded {
    value: String,
    /// Where the content was stored; `None` when it was only dropped from
    /// the row (a delete).
    key: Option<DocumentKey>,
}

impl Offloaded {
    pub(crate) fn key(&self) -> Option<&DocumentKey> {
        self.key.as_ref()
    }
}

/// Prepares `resource` for its database row. Any client-sent marker is
/// removed. With storage configured, content of at least the configured size
/// is uploaded and replaced by the marker, and a deleted version's content is
/// dropped (its earlier version keeps it). Returns what was taken off.
///
/// `resource` must already carry its id and `meta.versionId`.
pub(crate) async fn offload(
    storage: Option<&BinaryStorage>,
    tenant: &TenantId,
    project: &ProjectId,
    resource: &mut Resource,
    deleted: bool,
) -> Result<Option<Offloaded>, OperationOutcomeError> {
    let Resource::Binary(binary) = resource else {
        return Ok(None);
    };
    remove_marker(binary);

    let Some(storage) = storage else {
        return Ok(None);
    };
    let Binary {
        id,
        meta,
        contentType: content_type,
        data,
        ..
    } = binary;
    let Some(data) = data.as_deref_mut() else {
        return Ok(None);
    };

    if deleted {
        return Ok(data
            .value
            .take()
            .map(|value| Offloaded { value, key: None }));
    }

    let Some(value) = data.value.as_deref() else {
        return Ok(None);
    };
    // Base64 never decodes to more than three bytes per four characters.
    if value.len() / 4 * 3 < storage.min_bytes {
        return Ok(None);
    }
    // Content that does not decode is left in the row, as before.
    let Some(bytes) = decode(value) else {
        return Ok(None);
    };
    if bytes.len() < storage.min_bytes {
        return Ok(None);
    }

    let key = document_key(
        tenant,
        project,
        id.as_deref().unwrap_or_default(),
        version_id(meta.as_deref()).unwrap_or_default(),
    )?;
    let mut document = Document::new(bytes);
    if let Some(content_type) = content_type.value.as_deref() {
        document = document.with_content_type(content_type);
    }
    storage.storage.put(&key, document).await?;

    let value = data.value.take().unwrap_or_default();
    data.extension.get_or_insert_with(Vec::new).push(Extension {
        url: STORED_CONTENT_EXTENSION_URL.to_string(),
        value: Some(ExtensionValueTypeChoice::Boolean(Box::new(FHIRBoolean {
            value: Some(true),
            ..Default::default()
        }))),
        ..Default::default()
    });

    Ok(Some(Offloaded {
        value,
        key: Some(key),
    }))
}

/// Puts what [`offload`] took back on `resource`.
pub(crate) fn restore(resource: &mut Resource, offloaded: Option<Offloaded>) {
    let (Resource::Binary(binary), Some(offloaded)) = (resource, offloaded) else {
        return;
    };
    remove_marker(binary);
    binary.data.get_or_insert_with(Box::default).value = Some(offloaded.value);
}

/// Removes a document whose row was never written. Failure is only logged:
/// the orphan sweep removes it later.
pub(crate) async fn discard(storage: Option<&BinaryStorage>, offloaded: Option<&Offloaded>) {
    if let (Some(storage), Some(key)) = (storage, offloaded.and_then(Offloaded::key))
        && let Err(error) = storage.storage.delete(key).await
    {
        tracing::warn!(%key, %error, "could not remove the document of a failed Binary write");
    }
}

/// Puts the stored content back on every Binary in `resources` that carries
/// the marker.
pub(crate) async fn rehydrate<'a>(
    storage: Option<&BinaryStorage>,
    tenant: &TenantId,
    project: &ProjectId,
    resources: impl Iterator<Item = &'a mut Resource>,
) -> Result<(), OperationOutcomeError> {
    let stored = resources
        .filter_map(|resource| match resource {
            Resource::Binary(binary) if has_marker(binary) => Some(binary),
            _ => None,
        })
        .collect::<Vec<_>>();
    if stored.is_empty() {
        return Ok(());
    }

    let Some(storage) = storage else {
        return Err(OperationOutcomeError::fatal(
            IssueType::exception(),
            "Binary content is kept in document storage, which is not configured.".to_string(),
        ));
    };

    // Built before the stream so the stream's type holds no closure, which
    // keeps the caller's future `Send`.
    let fetches = stored
        .into_iter()
        .map(|binary| fetch(storage, tenant, project, binary))
        .collect::<Vec<_>>();
    stream::iter(fetches)
        .buffer_unordered(FETCH_CONCURRENCY)
        .try_collect::<()>()
        .await
}

async fn fetch(
    storage: &BinaryStorage,
    tenant: &TenantId,
    project: &ProjectId,
    binary: &mut Binary,
) -> Result<(), OperationOutcomeError> {
    let id = binary.id.as_deref().unwrap_or_default();
    let version_id = version_id(binary.meta.as_deref()).unwrap_or_default();
    let key = document_key(tenant, project, id, version_id)?;

    let bytes = match storage.storage.get(&key).await {
        Ok(document) => document.body.collect().await?,
        Err(StorageError::NotFound(_)) => {
            tracing::error!(%key, "stored Binary content is missing");
            return Err(OperationOutcomeError::fatal(
                IssueType::exception(),
                format!("The content of Binary/{id} (version {version_id}) is missing."),
            ));
        }
        Err(error) => return Err(error.into()),
    };

    remove_marker(binary);
    binary.data.get_or_insert_with(Box::default).value = Some(STANDARD.encode(&bytes));
    Ok(())
}

fn version_id(meta: Option<&Meta>) -> Option<&str> {
    meta?.versionId.as_ref()?.value.as_deref()
}

fn is_marker(extension: &Extension) -> bool {
    extension.url == STORED_CONTENT_EXTENSION_URL
}

fn has_marker(binary: &Binary) -> bool {
    binary
        .data
        .as_ref()
        .and_then(|data| data.extension.as_ref())
        .is_some_and(|extensions| extensions.iter().any(is_marker))
}

/// Removes the marker from `data`, and `data` itself when nothing else is
/// left on it.
fn remove_marker(binary: &mut Binary) {
    let Some(data) = binary.data.as_deref_mut() else {
        return;
    };
    if let Some(extensions) = data.extension.as_mut() {
        extensions.retain(|extension| !is_marker(extension));
        if extensions.is_empty() {
            data.extension = None;
        }
    }
    if data.id.is_none() && data.extension.is_none() && data.value.is_none() {
        binary.data = None;
    }
}

/// Decodes base64 content, which FHIR allows to contain whitespace.
fn decode(value: &str) -> Option<Vec<u8>> {
    if value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        let compact = value
            .bytes()
            .filter(|byte| !byte.is_ascii_whitespace())
            .collect::<Vec<_>>();
        STANDARD.decode(compact).ok()
    } else {
        STANDARD.decode(value).ok()
    }
}
