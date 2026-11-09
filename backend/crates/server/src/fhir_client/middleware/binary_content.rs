//! Keeps large `Binary.data` in document storage; see
//! [`crate::document_storage`].
//!
//! On the way in, a Binary whose content is at least the configured size has
//! the content uploaded and replaced by the marker, so storage writes a
//! row-sized Binary; the response gets the content back without a fetch. On
//! the way out, every Binary carrying the marker gets its content fetched,
//! unless `_summary` or `_elements` drop `data` anyway. A marker a client
//! sends is removed: only the server sets one. A patch is applied here, to
//! the Binary with its content, and sent on as an update.
//!
//! Every storage call names the request's tenant and project as the
//! document's scope, and the marker holds only a name within it, so stored
//! content is only ever looked up among the project's own documents.
//!
//! The document is uploaded before the row is written. When the write is
//! refused (a 4xx) the document is removed again; when it fails (a 5xx) the
//! row may still have landed, so the document is left for the sweep to judge.

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_util::{StreamExt as _, TryStreamExt as _, stream};
use haste_document_storage::{Document, DocumentName, DocumentScope, StorageError};
use haste_fhir_client::request::{
    FHIRPatchResponse, FHIRRequest, FHIRResponse, FHIRUpdateInstanceRequest, HistoryResponse,
    SearchResponse, UpdateRequest,
};
use haste_fhir_model::r4::generated::{
    resources::{Binary, Bundle, Resource, ResourceType},
    terminology::IssueType,
    types::{Extension, ExtensionValueTypeChoice, FHIRString},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};

use crate::{
    document_storage::{BinaryStorage, MARKER_URL, new_document_name},
    fhir_client::{
        ClientState, ServerCTX,
        middleware::{
            Backend, Middleware, Next, Output,
            element_filtering::keeps_element,
            storage::{patch_resource, read_patch_target},
        },
    },
};

/// Documents fetched at once when a response holds several stored Binaries.
const FETCH_CONCURRENCY: usize = 8;

/// Moves large `Binary.data` to document storage on the way in and back on
/// the way out.
pub struct BinaryContent;

impl<B: Backend> Middleware<B> for BinaryContent {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            match request {
                // Storage would patch the row form. Patch the Binary with its
                // content instead and store the result as an update.
                FHIRRequest::Patch(patch) if patch.resource_type == ResourceType::Binary => {
                    let mut current =
                        read_patch_target(state.repo.as_ref(), &ctx.tenant, &ctx.project, &patch)
                            .await?;
                    rehydrate(
                        state.binary_storage.as_ref(),
                        &ctx.tenant,
                        &ctx.project,
                        vec![&mut current],
                    )
                    .await?;
                    let patched = patch_resource(&current, &patch)?;
                    let update =
                        FHIRRequest::Update(UpdateRequest::Instance(FHIRUpdateInstanceRequest {
                            resource_type: patch.resource_type,
                            id: patch.id,
                            resource: patched,
                        }));
                    match write(state, ctx, update, next).await? {
                        FHIRResponse::Update(update) => {
                            Ok(FHIRResponse::Patch(FHIRPatchResponse {
                                resource: update.resource,
                            }))
                        }
                        other => Ok(other),
                    }
                }
                request @ (FHIRRequest::Create(_) | FHIRRequest::Update(_)) => {
                    write(state, ctx, request, next).await
                }
                request => read(state, ctx, request, next).await,
            }
        })
    }
}

/// Runs a create or update. A Binary's large content goes to storage first
/// and comes back on the response.
async fn write<'a, B: Backend>(
    state: &'a ClientState<B>,
    ctx: Arc<ServerCTX<B>>,
    mut request: FHIRRequest,
    next: Next<'a, B>,
) -> Result<FHIRResponse, OperationOutcomeError> {
    let storage = state.binary_storage.as_ref();
    let offloaded = match request_resource_mut(&mut request) {
        Some(resource) => offload(storage, &ctx.tenant, &ctx.project, resource).await?,
        None => None,
    };

    match next.run(state, ctx, request).await {
        Ok(mut response) => {
            if let Some(resource) = response_resources_mut(&mut response).into_iter().next() {
                restore(resource, offloaded);
            }
            Ok(response)
        }
        Err(error) => {
            if !error.is_server_error() {
                discard(storage, offloaded.as_ref()).await;
            }
            Err(error)
        }
    }
}

/// Runs any other request and puts stored content back on the Binaries of
/// its response.
async fn read<'a, B: Backend>(
    state: &'a ClientState<B>,
    ctx: Arc<ServerCTX<B>>,
    request: FHIRRequest,
    next: Next<'a, B>,
) -> Result<FHIRResponse, OperationOutcomeError> {
    // Decided from the request: once `_summary` or `_elements` drop `data`,
    // fetching it would be wasted.
    let wanted = keeps_element(&request, "data");
    let mut response = next.run(state, ctx.clone(), request).await?;
    if wanted {
        rehydrate(
            state.binary_storage.as_ref(),
            &ctx.tenant,
            &ctx.project,
            response_resources_mut(&mut response),
        )
        .await?;
    }
    Ok(response)
}

fn request_resource_mut(request: &mut FHIRRequest) -> Option<&mut Resource> {
    match request {
        FHIRRequest::Create(create) => Some(&mut create.resource),
        FHIRRequest::Update(UpdateRequest::Instance(update)) => Some(&mut update.resource),
        FHIRRequest::Update(UpdateRequest::Conditional(update)) => Some(&mut update.resource),
        _ => None,
    }
}

/// Every resource a response carries.
fn response_resources_mut(response: &mut FHIRResponse) -> Vec<&mut Resource> {
    match response {
        FHIRResponse::Create(create) => vec![&mut create.resource],
        FHIRResponse::Update(update) => vec![&mut update.resource],
        FHIRResponse::Patch(patch) => vec![&mut patch.resource],
        FHIRResponse::VersionRead(vread) => vec![&mut vread.resource],
        FHIRResponse::Read(read) => read.resource.iter_mut().collect(),
        FHIRResponse::Search(SearchResponse::Type(search)) => {
            bundle_resources_mut(&mut search.bundle)
        }
        FHIRResponse::Search(SearchResponse::System(search)) => {
            bundle_resources_mut(&mut search.bundle)
        }
        FHIRResponse::History(HistoryResponse::Instance(history)) => {
            bundle_resources_mut(&mut history.bundle)
        }
        FHIRResponse::History(HistoryResponse::Type(history)) => {
            bundle_resources_mut(&mut history.bundle)
        }
        FHIRResponse::History(HistoryResponse::System(history)) => {
            bundle_resources_mut(&mut history.bundle)
        }
        // The entries of a batch or transaction went through a pipeline of
        // their own.
        FHIRResponse::Delete(_)
        | FHIRResponse::Capabilities(_)
        | FHIRResponse::Invoke(_)
        | FHIRResponse::Accepted(_)
        | FHIRResponse::Batch(_)
        | FHIRResponse::Transaction(_) => Vec::new(),
    }
}

fn bundle_resources_mut(bundle: &mut Bundle) -> Vec<&mut Resource> {
    bundle
        .entry
        .iter_mut()
        .flatten()
        .filter_map(|entry| entry.resource.as_deref_mut())
        .collect()
}

/// Content taken off a Binary for its row, to be put back on the response.
struct Offloaded {
    value: String,
    scope: DocumentScope,
    name: DocumentName,
}

/// Prepares `resource` for its row. Any client-sent marker is removed. With
/// storage configured, content of at least the configured size is uploaded
/// and replaced by the marker. Returns what was taken off.
async fn offload(
    storage: Option<&BinaryStorage>,
    tenant: &TenantId,
    project: &ProjectId,
    resource: &mut Resource,
) -> Result<Option<Offloaded>, OperationOutcomeError> {
    let Resource::Binary(binary) = resource else {
        return Ok(None);
    };
    remove_marker(binary);

    let Some(storage) = storage else {
        return Ok(None);
    };
    let Binary {
        contentType: content_type,
        data,
        ..
    } = binary;
    let Some(data) = data.as_deref_mut() else {
        return Ok(None);
    };
    let Some(value) = data.value.as_deref() else {
        return Ok(None);
    };
    // Base64 never decodes to more than three bytes per four characters.
    if value.len() / 4 * 3 < storage.min_bytes() {
        return Ok(None);
    }
    // Content that does not decode stays in the row, as before.
    let Some(bytes) = decode(value) else {
        return Ok(None);
    };
    if bytes.len() < storage.min_bytes() {
        return Ok(None);
    }

    let scope = DocumentScope::new(tenant, project)?;
    let name = new_document_name()?;
    let mut document = Document::new(bytes);
    if let Some(content_type) = content_type.value.as_deref() {
        document = document.with_content_type(content_type);
    }
    storage.storage().put(&scope, &name, document).await?;

    let value = data.value.take().unwrap_or_default();
    data.extension
        .get_or_insert_with(Vec::new)
        .push(marker(&name));

    Ok(Some(Offloaded { value, scope, name }))
}

/// Puts what [`offload`] took back on `resource`.
fn restore(resource: &mut Resource, offloaded: Option<Offloaded>) {
    let (Resource::Binary(binary), Some(offloaded)) = (resource, offloaded) else {
        return;
    };
    remove_marker(binary);
    binary.data.get_or_insert_with(Box::default).value = Some(offloaded.value);
}

/// Removes the document of a write that was refused. Failure is only logged:
/// the sweep removes the document later.
async fn discard(storage: Option<&BinaryStorage>, offloaded: Option<&Offloaded>) {
    if let (Some(storage), Some(offloaded)) = (storage, offloaded)
        && let Err(error) = storage
            .storage()
            .delete(&offloaded.scope, &offloaded.name)
            .await
    {
        tracing::warn!(scope = %offloaded.scope, name = %offloaded.name, %error, "could not remove the document of a refused Binary write");
    }
}

/// Puts the stored content back on every Binary in `resources` that carries
/// the marker.
async fn rehydrate(
    storage: Option<&BinaryStorage>,
    tenant: &TenantId,
    project: &ProjectId,
    resources: Vec<&mut Resource>,
) -> Result<(), OperationOutcomeError> {
    let stored = resources
        .into_iter()
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
            "Binary content is kept in document storage, which is turned off \
             ([document_storage] backend = \"none\")."
                .to_string(),
        ));
    };

    let scope = DocumentScope::new(tenant, project)?;
    // Built before the stream so the stream's type holds no closure, which
    // keeps the caller's future `Send`.
    let fetches = stored
        .into_iter()
        .map(|binary| fetch(storage, &scope, binary))
        .collect::<Vec<_>>();
    stream::iter(fetches)
        .buffer_unordered(FETCH_CONCURRENCY)
        .try_collect::<()>()
        .await
}

/// Fetches the document the marker names from the request's `scope`. A name
/// the layer did not write can at most miss within that scope.
async fn fetch(
    storage: &BinaryStorage,
    scope: &DocumentScope,
    binary: &mut Binary,
) -> Result<(), OperationOutcomeError> {
    let Some(name) = marker_name(binary).and_then(|name| DocumentName::new(name).ok()) else {
        tracing::error!("stored Binary content has no usable document name");
        return Err(missing_content(binary));
    };

    let bytes = match storage.storage().get(scope, &name).await {
        Ok(document) => document.body.collect().await?,
        Err(StorageError::NotFound(_)) => {
            tracing::error!(%scope, %name, "stored Binary content is missing");
            return Err(missing_content(binary));
        }
        Err(error) => return Err(error.into()),
    };

    remove_marker(binary);
    binary.data.get_or_insert_with(Box::default).value = Some(STANDARD.encode(&bytes));
    Ok(())
}

fn missing_content(binary: &Binary) -> OperationOutcomeError {
    let id = binary.id.as_deref().unwrap_or_default();
    let version_id = binary
        .meta
        .as_deref()
        .and_then(|meta| meta.versionId.as_deref())
        .and_then(|version| version.value.as_deref())
        .unwrap_or_default();
    OperationOutcomeError::fatal(
        IssueType::exception(),
        format!("The content of Binary/{id} (version {version_id}) is missing."),
    )
}

fn marker(name: &DocumentName) -> Extension {
    Extension {
        url: MARKER_URL.to_string(),
        value: Some(ExtensionValueTypeChoice::String(Box::new(FHIRString {
            value: Some(name.to_string()),
            ..Default::default()
        }))),
        ..Default::default()
    }
}

fn is_marker(extension: &Extension) -> bool {
    extension.url == MARKER_URL
}

fn markers(binary: &Binary) -> impl Iterator<Item = &Extension> {
    binary
        .data
        .as_ref()
        .and_then(|data| data.extension.as_ref())
        .into_iter()
        .flatten()
        .filter(|extension| is_marker(extension))
}

fn has_marker(binary: &Binary) -> bool {
    markers(binary).next().is_some()
}

/// The name the marker carries, or `None` without a marker or with one that
/// carries no string.
fn marker_name(binary: &Binary) -> Option<&str> {
    markers(binary).find_map(|extension| match &extension.value {
        Some(ExtensionValueTypeChoice::String(name)) => name.value.as_deref(),
        _ => None,
    })
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
