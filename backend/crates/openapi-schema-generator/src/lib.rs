use std::collections::{HashMap, HashSet};

use haste_fhir_model::r4::generated::{
    resources::{SearchParameter, StructureDefinition},
    terminology::{IssueType, SearchParamType, StructureDefinitionKind},
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct OpenAPIComponents {
    schemas: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIOperationContent {
    description: String,
    // Content Type to Schema mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIOperation {
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    request_body: Option<OpenAPIOperationContent>,
    responses: HashMap<String, OpenAPIOperationContent>,
    parameters: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIPathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<OpenAPIOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<OpenAPIOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<OpenAPIOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<OpenAPIOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<OpenAPIOperation>,
}

pub type OpenAPIPaths = HashMap<String, OpenAPIPathItem>;

#[derive(Deserialize, Serialize)]
pub struct OpenAPIInfo {
    title: String,
    version: String,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIServerVariable {
    default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIServer {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    variables: HashMap<String, OpenAPIServerVariable>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPI {
    servers: Vec<OpenAPIServer>,
    openapi: String,
    info: OpenAPIInfo,
    components: OpenAPIComponents,
    paths: OpenAPIPaths,
}

fn resource_schema(resource_name: &str) -> serde_json::Value {
    json!({
        "$ref": format!("#/components/schemas/{resource_name}")
    })
}

fn operation_outcome_schema() -> serde_json::Value {
    json!({
        "$ref": "#/components/schemas/OperationOutcome"
    })
}

fn json_content(schema: serde_json::Value) -> HashMap<String, serde_json::Value> {
    HashMap::from([("application/json".to_string(), schema)])
}

fn response(
    description: impl Into<String>,
    content: Option<HashMap<String, serde_json::Value>>,
) -> OpenAPIOperationContent {
    OpenAPIOperationContent {
        description: description.into(),
        content,
    }
}

fn resource_response(
    resource_name: &str,
    description: impl Into<String>,
) -> OpenAPIOperationContent {
    response(
        description,
        Some(json_content(json!({
            "schema": resource_schema(resource_name)
        }))),
    )
}

fn operation_outcome_response(description: impl Into<String>) -> OpenAPIOperationContent {
    response(
        description,
        Some(json_content(json!({
            "schema": operation_outcome_schema()
        }))),
    )
}

fn id_parameter(resource_name: &str) -> serde_json::Value {
    json!({
        "name": "id",
        "in": "path",
        "required": true,
        "schema": {
            "type": "string"
        },
        "description": format!("The ID of the {resource_name} resource")
    })
}

fn read_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: None,
        responses: HashMap::from([
            (
                "200".to_string(),
                resource_response(
                    resource_name,
                    format!("Successful read of {resource_name} resource"),
                ),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
            (
                "500".to_string(),
                operation_outcome_response("Server error"),
            ),
        ]),
        parameters: vec![id_parameter(resource_name)],
    }
}

fn put_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: Some(resource_response(
            resource_name,
            format!("The {resource_name} resource to create or update"),
        )),
        responses: HashMap::from([
            (
                "200".to_string(),
                resource_response(
                    resource_name,
                    format!("Successful put/creation of {resource_name} resource"),
                ),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
            (
                "500".to_string(),
                operation_outcome_response("Server error"),
            ),
        ]),
        parameters: vec![id_parameter(resource_name)],
    }
}

fn delete_instance_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: None,
        responses: HashMap::from([
            (
                "200".to_string(),
                response(
                    format!("Successful deletion of {resource_name} resource"),
                    None,
                ),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
        ]),
        parameters: vec![id_parameter(resource_name)],
    }
}

fn patch_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: Some(OpenAPIOperationContent {
            description: format!("JSON Patch operation for {resource_name} resource."),
            content: Some(json_content(json!({
                "schema": {
                    "type": "array"
                }
            }))),
        }),
        responses: HashMap::from([
            (
                "200".to_string(),
                resource_response(
                    resource_name,
                    format!("Successful patch of {resource_name} resource"),
                ),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
        ]),
        parameters: vec![id_parameter(resource_name)],
    }
}

fn create_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: Some(resource_response(
            resource_name,
            format!("The {resource_name} resource to create"),
        )),
        responses: HashMap::from([
            (
                "200".to_string(),
                resource_response(
                    resource_name,
                    format!("Successful creation of {resource_name} resource"),
                ),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
        ]),
        parameters: vec![],
    }
}

fn search_resource_operation(
    resource_name: &str,
    parameters: Vec<serde_json::Value>,
) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: None,
        responses: HashMap::from([
            (
                "200".to_string(),
                response(
                    "Successful search operation",
                    Some(json_content(json!({
                        "schema": haste_sd_to_json_schema::bundle_of_resource(&json!({
                            "$ref": format!("#/components/schemas/{resource_name}")
                        }))
                    }))),
                ),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
        ]),
        parameters,
    }
}

fn delete_resource_operation(parameters: Vec<serde_json::Value>) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: None,
        responses: HashMap::from([
            (
                "200".to_string(),
                response("Successful delete operation", None),
            ),
            (
                "400".to_string(),
                operation_outcome_response("Client error"),
            ),
        ]),
        parameters,
    }
}

fn resource_search_parameters_schema(
    resource_name: &str,
    search_parameters: &[SearchParameter],
) -> Vec<serde_json::Value> {
    search_parameters
        .iter()
        .filter(|sp| {
            sp.base.iter().any(|b| {
                let base = b.as_str();

                base == Some(resource_name)
                    || base == Some("Resource")
                    || base == Some("DomainResource")
            }) && sp.type_ != SearchParamType::composite()
        })
        .map(|sp| {
            let search_type = if sp.type_ == SearchParamType::number() {
                "number"
            } else {
                "string"
            };

            json!({
                "name": sp.code.value,
                "in": "query",
                "required": false,
                "schema": {
                    "type": search_type
                },
                "description": sp.description.value.as_deref().unwrap_or_default()
            })
        })
        .collect()
}

/// Generates an [`OpenAPI`] document containing only the FHIR resources supported by the server.
///
/// Resource and complex-type schemas are represented as external `$ref`s pointing to
/// `{schema_base_url}/{TypeName}`. This keeps the generated document small while allowing
/// individual schemas to be served on demand from a separate endpoint.
///
/// For each supported resource, the generated document includes endpoints for reading,
/// creating, searching, updating, and deleting resources, as applicable.
///
/// # Arguments
///
/// * `server_root` - The root URL of the FHIR server.
/// * `api_version` - The version of the API represented by the generated document.
/// * `schema_base_url` - The base URL from which individual FHIR schemas are served
///   (e.g. `/schemas/fhir`).
/// * `sds` - All available [`StructureDefinition`]s, including resource and complex-type
///   definitions.
/// * `search_parameters` - All available [`SearchParameter`]s used to generate resource
///   search operations.
/// * `supported_resource_names` - The set of FHIR resource type names supported by the
///   server. Resource definitions not present in this set are excluded from the generated
///   document.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] if a supported resource
/// [`StructureDefinition`] does not contain a type name.
///
/// # Returns
///
/// An [`OpenAPI`] document containing the server configuration, schemas, and paths for
/// all supported resources and applicable complex types.
pub fn open_api_schema_generator<S: std::hash::BuildHasher>(
    server_root: &str,
    api_version: &str,
    schema_base_url: &str,
    sds: &[StructureDefinition],
    search_parameters: &[SearchParameter],
    supported_resource_names: &HashSet<String, S>,
) -> Result<OpenAPI, OperationOutcomeError> {
    let mut openapi_schema = create_openapi_schema(server_root, api_version);

    add_resource_schemas(
        &mut openapi_schema,
        sds,
        search_parameters,
        supported_resource_names,
        schema_base_url,
    )?;

    add_complex_type_schemas(&mut openapi_schema, sds, schema_base_url);

    Ok(openapi_schema)
}

fn create_openapi_schema(server_root: &str, api_version: &str) -> OpenAPI {
    let mut fhir_server_variables = HashMap::new();

    fhir_server_variables.insert(
        "tenant".to_string(),
        OpenAPIServerVariable {
            default: "my-tenant".to_string(),
            description: Some("Tenant identifier".to_string()),
        },
    );

    fhir_server_variables.insert(
        "project".to_string(),
        OpenAPIServerVariable {
            default: "my-project".to_string(),
            description: Some("Project identifier".to_string()),
        },
    );

    fhir_server_variables.insert(
        "fhir_version".to_string(),
        OpenAPIServerVariable {
            default: "r4".to_string(),
            description: Some("FHIR version".to_string()),
        },
    );

    OpenAPI {
        openapi: "3.1.1".to_string(),
        servers: vec![OpenAPIServer {
            url: format!(
                "{}/w/{}/{}/api/v1/fhir/{}",
                server_root, "{tenant}", "{project}", "{fhir_version}"
            ),
            description: Some("Haste Health FHIR Server".to_string()),
            variables: fhir_server_variables,
        }],
        info: OpenAPIInfo {
            title: "Haste Health API Documentation".to_string(),
            version: api_version.to_string(),
        },
        components: OpenAPIComponents {
            schemas: HashMap::new(),
        },
        paths: HashMap::new(),
    }
}

fn add_resource_schemas<S: std::hash::BuildHasher>(
    openapi_schema: &mut OpenAPI,
    sds: &[StructureDefinition],
    search_parameters: &[SearchParameter],
    supported_resource_names: &HashSet<String, S>,
    schema_base_url: &str,
) -> Result<(), OperationOutcomeError> {
    let resource_sds = sds
        .iter()
        .filter(|sd| sd.kind == StructureDefinitionKind::resource())
        .filter(|sd| {
            sd.type_
                .value
                .as_ref()
                .is_some_and(|name| supported_resource_names.contains(name))
        });

    for sd in resource_sds {
        let resource_name = sd.type_.value.as_ref().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::structure(),
                format!(
                    "StructureDefinition missing type for id {}",
                    sd.id.as_ref().unwrap_or(&"unknown".to_string())
                ),
            )
        })?;

        add_resource_schema(
            openapi_schema,
            resource_name,
            search_parameters,
            schema_base_url,
        );
    }

    Ok(())
}

fn add_resource_schema(
    openapi_schema: &mut OpenAPI,
    resource_name: &str,
    search_parameters: &[SearchParameter],
    schema_base_url: &str,
) {
    // Use external $ref for the resource schema — individual schemas are served
    // from a separate endpoint to keep the main document small.
    openapi_schema.components.schemas.insert(
        resource_name.to_string(),
        json!({
            "$ref": format!("{}/{}", schema_base_url, resource_name)
        }),
    );

    // Read operation
    openapi_schema.paths.insert(
        format!("/{resource_name}/{{id}}"),
        OpenAPIPathItem {
            get: Some(read_resource_operation(resource_name)),
            post: None,
            patch: Some(patch_resource_operation(resource_name)),
            put: Some(put_resource_operation(resource_name)),
            delete: Some(delete_instance_operation(resource_name)),
        },
    );

    // Search/create/delete operations
    let resource_search_parameters =
        resource_search_parameters_schema(resource_name, search_parameters);

    openapi_schema.paths.insert(
        format!("/{resource_name}"),
        OpenAPIPathItem {
            get: Some(search_resource_operation(
                resource_name,
                resource_search_parameters.clone(),
            )),
            patch: None,
            put: None,
            post: Some(create_resource_operation(resource_name)),
            delete: Some(delete_resource_operation(resource_search_parameters)),
        },
    );
}

fn add_complex_type_schemas(
    openapi_schema: &mut OpenAPI,
    sds: &[StructureDefinition],
    schema_base_url: &str,
) {
    // Complex types (datatypes) get the same external-$ref treatment as
    // resources — served on demand from `{schema_base_url}/{TypeName}`
    // rather than tracked/inlined here.
    //
    // "Element" is matched by name too since it's FHIR's abstract base type
    // and isn't always loaded with `kind: complex-type`.
    for sd in sds.iter().filter(|sd| {
        sd.kind == StructureDefinitionKind::complex_type()
            || sd.name.value.as_deref() == Some("Element")
    }) {
        let Some(type_name) = sd.type_.value.as_ref() else {
            continue;
        };

        openapi_schema.components.schemas.insert(
            type_name.clone(),
            json!({
                "$ref": format!("{}/{}", schema_base_url, type_name)
            }),
        );
    }
}

/// Returns the set of resource type names from the given
/// [`StructureDefinition`]s.
/// Useful for getting the full unfiltered set when no `CapabilityStatement`
/// filtering is desired.
pub fn all_resource_names(sds: &[StructureDefinition]) -> HashSet<String> {
    sds.iter()
        .filter(|sd| sd.kind == StructureDefinitionKind::resource())
        .filter_map(|sd| sd.type_.value.clone())
        .collect()
}
