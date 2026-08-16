use crate::{
    load_artifacts::{get_all_sds, get_all_sps},
    services::ServerState,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    response::Response,
};
use haste_fhir_model::r4::generated::terminology::{IssueType, StructureDefinitionKind};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_openapi_schema_generator::OpenAPI;
use haste_repository::Repository;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock},
};
use tokio::sync::Mutex;

/// Cached OpenAPI document. Generated once on first request.
static OPENAPI_DOCUMENT: LazyLock<Mutex<Option<OpenAPI>>> = LazyLock::new(|| Mutex::new(None));

/// Cached individual type schemas. Maps resource/complex type name -> JSON
/// schema string. Complex types (and primitives-adjacent references like
/// "Element") are referenced via external `$ref`s back into this same
/// endpoint rather than embedded, so each entry only needs its own shape.
static RESOURCE_SCHEMAS: LazyLock<Mutex<Option<HashMap<String, String>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Extract the set of supported resource type names from the loaded StructureDefinitions.
/// In the current architecture, every loaded resource SD is considered "supported".
fn get_supported_resource_names(
    sds: &[haste_fhir_model::r4::generated::resources::StructureDefinition],
) -> HashSet<String> {
    haste_openapi_schema_generator::all_resource_names(sds)
}

/// Builds each supported resource's and each complex type's JSON schema.
/// References to other complex types (and "Element") use external `$ref`s
/// pointing back at `{schema_base_url}/{TypeName}` — the same endpoint these
/// schemas are themselves served from — rather than being inlined, so there's
/// no need to track or embed transitively-referenced types.
fn build_resource_schemas(
    sds: &[haste_fhir_model::r4::generated::resources::StructureDefinition],
    supported_names: &HashSet<String>,
    schema_base_url: &str,
) -> Result<HashMap<String, String>, OperationOutcomeError> {
    let mut schemas = HashMap::new();

    // "Element" is matched by name too since it's FHIR's abstract base type
    // and isn't always loaded with `kind: complex-type`.
    let named_sds = sds.iter().filter(|sd| {
        (sd.kind == StructureDefinitionKind::resource()
            && sd
                .type_
                .value
                .as_ref()
                .is_some_and(|name| supported_names.contains(name)))
            || sd.kind == StructureDefinitionKind::complex_type()
            || sd.name.value.as_deref() == Some("Element")
    });

    for sd in named_sds {
        let type_name = sd
            .type_
            .value
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::structure(),
                    "StructureDefinition missing type".to_string(),
                )
            })?
            .clone();

        let schema = haste_sd_to_json_schema::isolated_schema(schema_base_url, sd)?;

        let json_str = serde_json::to_string(&schema).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to serialize schema for {}", type_name),
            )
        })?;

        schemas.insert(type_name, json_str);
    }

    Ok(schemas)
}

/// Ensures both the OpenAPI document and individual resource schemas are generated and cached.
/// Returns the lock guards for both caches.
async fn ensure_schemas_generated<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &Arc<ServerState<Repo, Search, Terminology>>,
) -> Result<(), OperationOutcomeError> {
    // Check if already generated (quick path)
    {
        let doc_guard = OPENAPI_DOCUMENT.lock().await;
        let schema_guard = RESOURCE_SCHEMAS.lock().await;
        if doc_guard.is_some() && schema_guard.is_some() {
            return Ok(());
        }
    }

    // Generate everything
    let sps = get_all_sps(state.repo.as_ref(), state.search.as_ref()).await?;
    let sds = get_all_sds(
        &["resource", "complex-type"],
        state.repo.as_ref(),
        state.search.as_ref(),
    )
    .await?;

    let api_url = &state.config.api_uri;
    let api_version = env!("CARGO_PKG_VERSION");
    let supported_names = get_supported_resource_names(&sds);

    // The schema_base_url points to the endpoint that serves individual resource schemas
    let schema_base_url = format!("{}/schemas/fhir", api_url);

    let openapi_document = haste_openapi_schema_generator::open_api_schema_generator(
        api_url,
        api_version,
        &schema_base_url,
        &sds,
        &sps,
        &supported_names,
    )?;

    let resource_schemas = build_resource_schemas(&sds, &supported_names, &schema_base_url)?;

    // Store both caches under their locks
    {
        let mut doc_lock = OPENAPI_DOCUMENT.lock().await;
        *doc_lock = Some(openapi_document);
    }

    {
        let mut schema_lock = RESOURCE_SCHEMAS.lock().await;
        *schema_lock = Some(resource_schemas);
    }

    Ok(())
}

/// Handler for `GET /openapi.json`
///
/// Returns the OpenAPI document with external `$ref`s for resource schemas.
/// The document is generated once and cached for the lifetime of the process.
pub async fn openapi_document_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    ensure_schemas_generated(&state).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let guard = OPENAPI_DOCUMENT.lock().await;
    if let Some(doc) = &*guard {
        Ok((
            headers,
            serde_json::to_string(doc).map_err(|_e| {
                OperationOutcomeError::error(
                    IssueType::exception(),
                    "Failed to serialize OpenAPI document".to_string(),
                )
            })?,
        )
            .into_response())
    } else {
        Err(OperationOutcomeError::error(
            IssueType::exception(),
            "OpenAPI document not available".to_string(),
        ))
    }
}

#[derive(Deserialize)]
pub struct SchemaPath {
    resource_type: String,
}

/// Handler for `GET /schemas/fhir/{resource_type}`
///
/// Returns the JSON Schema for a specific FHIR resource type or complex type
/// (datatype). References to other complex types are external `$ref`s back
/// into this same endpoint rather than inlined - this is the endpoint both
/// the main OpenAPI document's `components.schemas` entries and every
/// schema's own internal `$ref`s point to.
pub async fn resource_schema_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Path(path): Path<SchemaPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    ensure_schemas_generated(&state).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let guard = RESOURCE_SCHEMAS.lock().await;
    if let Some(schemas) = &*guard {
        if let Some(schema_json) = schemas.get(&path.resource_type) {
            Ok((headers, schema_json.clone()).into_response())
        } else {
            Err(OperationOutcomeError::error(
                IssueType::not_found(),
                format!("Schema not found for resource type: {}", path.resource_type),
            ))
        }
    } else {
        Err(OperationOutcomeError::error(
            IssueType::exception(),
            "Resource schemas not available".to_string(),
        ))
    }
}
