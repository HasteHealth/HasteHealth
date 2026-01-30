use std::collections::HashMap;

use haste_fhir_model::r4::generated::{
    resources::{SearchParameter, StructureDefinition},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct OpenAPIComponents {
    schemas: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct OpenAPIOperationResponse {
    description: String,
    // Content Type to Schema mapping
    content: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct OpenAPIOperation {
    responses: HashMap<String, OpenAPIOperationResponse>,
    parameters: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct OpenAPIPathItem {
    get: Option<OpenAPIOperation>,
    post: Option<OpenAPIOperation>,
    put: Option<OpenAPIOperation>,
    delete: Option<OpenAPIOperation>,
}

type OpenAPIPaths = HashMap<String, OpenAPIPathItem>;

#[derive(Deserialize, Serialize)]
struct OpenAPIInfo {
    title: String,
    version: String,
}

#[derive(Deserialize, Serialize)]
struct OpenAPIServerVariable {
    default: String,
    description: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct OpenAPIServer {
    url: String,
    description: Option<String>,
    variables: HashMap<String, OpenAPIServerVariable>,
}

#[derive(Deserialize, Serialize)]
struct OpenAPISchema {
    servers: Vec<OpenAPIServer>,
    openapi: String,
    info: OpenAPIInfo,
    components: OpenAPIComponents,
    paths: OpenAPIPaths,
}

fn open_api_schema_generator(
    server_root: &str,
    api_version: &str,
    resource_sds: &Vec<StructureDefinition>,
    search_parameters: &Vec<SearchParameter>,
) -> Result<OpenAPISchema, OperationOutcomeError> {
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
    let mut openapi_schema = OpenAPISchema {
        openapi: "3.0.1".to_string(),
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

    for sd in resource_sds {
        let json_schema = haste_sd_to_json_schema::resource_to_json_schema(sd)?;
        let resource_name = sd.type_.value.ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Structure(None),
                format!(
                    "StructureDefinition missing type for id {}",
                    sd.id.unwrap_or("unknown".to_string())
                ),
            )
        })?;
        // Read Operation
        openapi_schema.paths.insert(
            format!("/{}/{{id}}", resource_name),
            OpenAPIPathItem {
                get: Some(OpenAPIOperation {
                    responses: HashMap::new(
                        "200".to_string(),
                        OpenAPIOperationResponse {
                            description: todo!(),
                            content: HashMap::new(
                                "application/fhir+json".to_string(),
                                json_schema.clone(),
                            ),
                        },
                    ),
                    parameters: json!({
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "schema": {
                            "type": "string"
                        },
                        "description": format!("The ID of the {} resource", resource_name)
                    }),
                }),
                post: None,
                put: None,
                delete: None,
            },
        );

        openapi_schema
            .components
            .schemas
            .insert(resource_name, json_schema);
    }

    todo!();
}
