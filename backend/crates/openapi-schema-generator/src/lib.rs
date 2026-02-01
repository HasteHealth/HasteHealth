use std::collections::HashMap;

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

fn read_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: None,
        responses: HashMap::from([
            (
                "200".to_string(),
                OpenAPIOperationContent {
                    description: format!("Successful read of {} resource", resource_name),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": format!("#/components/schemas/{}", resource_name) }}),
                    )])),
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
            (
                "500".to_string(),
                OpenAPIOperationContent {
                    description: "Server error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
        ]),
        parameters: vec![json!({
            "name": "id",
            "in": "path",
            "required": true,
            "schema": {
                "type": "string"
            },
            "description": format!("The ID of the {} resource", resource_name)
        })],
    }
}

fn put_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: Some(OpenAPIOperationContent {
            description: format!("The {} resource to create or update", resource_name),
            content: Some(HashMap::from([(
                "application/json".to_string(),
                json!({ "schema": {"$ref": format!("#/components/schemas/{}", resource_name) }}),
            )])),
        }),
        responses: HashMap::from([
            (
                "200".to_string(),
                OpenAPIOperationContent {
                    description: format!("Successful put/creation of {} resource", resource_name),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": format!("#/components/schemas/{}", resource_name) }}),
                    )])),
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
            (
                "500".to_string(),
                OpenAPIOperationContent {
                    description: "Server error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
        ]),
        parameters: vec![json!({
            "name": "id",
            "in": "path",
            "required": true,
            "schema": {
                "type": "string"
            },
            "description": format!("The ID of the {} resource", resource_name)
        })],
    }
}

fn delete_instance_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: None,
        responses: HashMap::from([
            (
                "200".to_string(),
                OpenAPIOperationContent {
                    description: format!("Successful deletion of {} resource", resource_name),
                    content: None,
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
        ]),
        parameters: vec![json!({
            "name": "id",
            "in": "path",
            "required": true,
            "schema": {
                "type": "string"
            },
            "description": format!("The ID of the {} resource", resource_name)
        })],
    }
}

fn patch_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: Some(OpenAPIOperationContent {
            description: format!("JSON Patch operation for {} resource.", resource_name),
            content: Some(HashMap::from([(
                "application/json".to_string(),
                json!({ "schema": {"type": "array" }}),
            )])),
        }),
        responses: HashMap::from([
            (
                "200".to_string(),
                OpenAPIOperationContent {
                    description: format!("Successful patch of {} resource", resource_name),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": format!("#/components/schemas/{}", resource_name) }}),
                    )])),
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
        ]),
        parameters: vec![json!({
            "name": "id",
            "in": "path",
            "required": true,
            "schema": {
                "type": "string"
            },
            "description": format!("The ID of the {} resource", resource_name)
        })],
    }
}

fn resource_search_parameters_schema(
    resource_name: &str,
    search_parameters: &Vec<SearchParameter>,
) -> Vec<serde_json::Value> {
    let mut params = vec![];

    for sp in search_parameters.iter().filter(|sp| {
        sp.base.iter().any(|b| {
            let base: Option<String> = b.as_ref().into();
            let base = base.as_ref().map(|s| s.as_str());
            base == Some(resource_name)
                || base == Some("Resource")
                || base == Some("DomainResource")
        }) && !matches!(sp.type_.as_ref(), &SearchParamType::Composite(_))
    }) {
        let search_type = match sp.type_.as_ref() {
            SearchParamType::Quantity(_)
            | SearchParamType::Special(_)
            | SearchParamType::Token(_)
            | SearchParamType::Uri(_)
            | SearchParamType::Null(_)
            | SearchParamType::Reference(_)
            | SearchParamType::Composite(_)
            | SearchParamType::Date(_)
            | SearchParamType::String(_) => "string",

            SearchParamType::Number(_) => "number",
        };

        params.push(json!({
            "name": sp.code.value,
            "in": "query",
            "required": false,
            "schema": {
                "type": search_type
            },
            "description": sp.description.value.as_ref().map(|s| s.as_str()).unwrap_or("")
        }));
    }

    params
}

fn create_resource_operation(resource_name: &str) -> OpenAPIOperation {
    OpenAPIOperation {
        request_body: Some(OpenAPIOperationContent {
            description: format!("The {} resource to create", resource_name),
            content: Some(HashMap::from([(
                "application/json".to_string(),
                json!({ "schema": {"$ref": format!("#/components/schemas/{}", resource_name) }}),
            )])),
        }),
        responses: HashMap::from([
            (
                "200".to_string(),
                OpenAPIOperationContent {
                    description: format!("Successful creation of {} resource", resource_name),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": format!("#/components/schemas/{}", resource_name) }}),
                    )])),
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
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
                OpenAPIOperationContent {
                    description: "Successful search operation".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": haste_sd_to_json_schema::bundle_of_resource(json!({
                            "$ref": format!("#/components/schemas/{}", resource_name)
                        })) }),
                    )])),
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
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
                OpenAPIOperationContent {
                    description: "Successful delete operation".to_string(),
                    content: None,
                },
            ),
            (
                "400".to_string(),
                OpenAPIOperationContent {
                    description: "Client error".to_string(),
                    content: Some(HashMap::from([(
                        "application/json".to_string(),
                        json!({ "schema": {"$ref": "#/components/schemas/OperationOutcome" }}),
                    )])),
                },
            ),
        ]),
        parameters,
    }
}

pub fn open_api_schema_generator(
    server_root: &str,
    api_version: &str,
    sds: &Vec<StructureDefinition>,
    search_parameters: &Vec<SearchParameter>,
) -> Result<OpenAPI, OperationOutcomeError> {
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
    let mut openapi_schema = OpenAPI {
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
    };

    let complex_sds = sds.iter().filter(|sd| match sd.kind.as_ref() {
        StructureDefinitionKind::ComplexType(_) => true,
        _ => false,
    });

    for sd in complex_sds {
        let json_schema = haste_sd_to_json_schema::isolated_schema("#/components/schemas", sd)?;
        let type_name = sd.type_.value.as_ref().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Structure(None),
                format!(
                    "StructureDefinition missing type for id {}",
                    sd.id.as_ref().unwrap_or(&"unknown".to_string())
                ),
            )
        })?;
        openapi_schema
            .components
            .schemas
            .insert(type_name.clone(), json_schema);
    }

    let resource_sds = sds.iter().filter(|sd| match sd.kind.as_ref() {
        StructureDefinitionKind::Resource(_) => true,
        _ => false,
    });

    for sd in resource_sds {
        let json_schema = haste_sd_to_json_schema::isolated_schema("#/components/schemas", sd)?;
        let resource_name = sd.type_.value.as_ref().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Structure(None),
                format!(
                    "StructureDefinition missing type for id {}",
                    sd.id.as_ref().unwrap_or(&"unknown".to_string())
                ),
            )
        })?;

        // Read Operation
        openapi_schema.paths.insert(
            format!("/{}/{{id}}", resource_name),
            OpenAPIPathItem {
                get: Some(read_resource_operation(&resource_name)),
                post: None,
                patch: Some(patch_resource_operation(&resource_name)),
                put: Some(put_resource_operation(&resource_name)),
                delete: Some(delete_instance_operation(&resource_name)),
            },
        );

        let resource_search_parameters =
            resource_search_parameters_schema(&resource_name, search_parameters);

        openapi_schema.paths.insert(
            format!("/{}", resource_name),
            OpenAPIPathItem {
                get: Some(search_resource_operation(
                    &resource_name,
                    resource_search_parameters.clone(),
                )),
                patch: None,
                put: None,
                post: Some(create_resource_operation(&resource_name)),
                delete: Some(delete_resource_operation(resource_search_parameters)),
            },
        );

        openapi_schema
            .components
            .schemas
            .insert(resource_name.clone(), json_schema);
    }

    openapi_schema.components.schemas.insert(
        "Element".to_string(),
        json!({
            "additionalProperties": false,
            "properties": {
                "extension": {
                    "items": {
                        "$ref": "#/components/schemas/Extension"
                    },
                    "type": "array"
                },
                "id": {
                    "type": "string"
                }
            },
            "required": [],
            "type": "object"
        }),
    );

    Ok(openapi_schema)
}
