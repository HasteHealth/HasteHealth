use std::collections::HashMap;

use haste_fhir_model::r4::generated::{
    resources::{SearchParameter, StructureDefinition},
    terminology::{IssueType, StructureDefinitionKind},
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct OpenAPIComponents {
    schemas: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIOperationResponse {
    description: String,
    // Content Type to Schema mapping
    content: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
pub struct OpenAPIOperation {
    responses: HashMap<String, OpenAPIOperationResponse>,
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
        responses: HashMap::from([(
            "200".to_string(),
            OpenAPIOperationResponse {
                description: format!("Successful read of {} resource", resource_name),
                content: HashMap::from([(
                    "application/fhir+json".to_string(),
                    json!({ "$ref": format!("#/components/schemas/{}", resource_name) }),
                )]),
            },
        )]),
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

pub fn open_api_schema_generator(
    server_root: &str,
    api_version: &str,
    sds: &Vec<StructureDefinition>,
    _search_parameters: &Vec<SearchParameter>,
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
                put: None,
                delete: None,
            },
        );

        openapi_schema
            .components
            .schemas
            .insert(resource_name.clone(), json_schema);
    }

    Ok(openapi_schema)
}
