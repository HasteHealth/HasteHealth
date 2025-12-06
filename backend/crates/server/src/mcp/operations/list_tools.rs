use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        request::ListToolsRequest,
        schemas::schema_2025_11_25::{ListToolsResult, Tool},
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{CapabilityStatementRestResource, CapabilityStatementRestResourceSearchParam},
    terminology::{SearchParamType, TypeRestfulInteraction},
};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use serde_json::json;
use std::sync::Arc;

pub fn search_tool_parameters(
    capability_search_params: &Vec<CapabilityStatementRestResourceSearchParam>,
) -> serde_json::Value {
    let mut properties: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    for capability_parameter in capability_search_params.iter() {
        let name = capability_parameter.name.value.clone().unwrap_or_default();
        let description = capability_parameter
            .documentation
            .as_ref()
            .and_then(|d| d.value.as_ref());

        let json_schema_type = match &*capability_parameter.type_ {
            SearchParamType::Number(_) => Some("number".to_string()),
            SearchParamType::Special(_)
            | SearchParamType::Quantity(_)
            | SearchParamType::Reference(_)
            | SearchParamType::Date(_)
            | SearchParamType::String(_)
            | SearchParamType::Token(_)
            | SearchParamType::Uri(_) => Some("string".to_string()),
            SearchParamType::Composite(_) | SearchParamType::Null(_) => None,
        };

        if let Some(json_schema_type) = json_schema_type {
            properties.insert(
                name,
                json!({
                    "type": json_schema_type,
                    "description": description,
                }),
            );
        }
    }

    serde_json::Value::Object(properties)
}

fn generate_resource_schema(
    capability_search_params: &CapabilityStatementRestResource,
) -> serde_json::Value {
    let operations_supported = capability_search_params
        .interaction
        .as_ref()
        .map(|interactions| {
            interactions
                .iter()
                .map(|interaction| interaction.code.as_ref())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut schema = json!({
        "properties": {
            "operation": {
                "type": "string",
                "enum": operations_supported.iter().filter_map(|op| {
                    (*op).into()
                }).collect::<Vec<String>>(),
            }
        },
        "required": ["operation"],
        "type": "object"
    });

    if operations_supported
        .iter()
        .find(|code| matches!(code, TypeRestfulInteraction::SearchType(_)))
        .is_some()
    {
        let search_properties = search_tool_parameters(
            capability_search_params
                .searchParam
                .as_ref()
                .unwrap_or(&vec![]),
        );

        schema["properties"]["search_parameters"] = json!({
            "type": "object",
            "properties": search_properties,
        });
    }

    schema
}

pub async fn list_tools<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    ctx: Arc<ServerCTX<Repo, Search, Terminology>>,
    _request: &ListToolsRequest,
) -> Result<ListToolsResult, MCPError<serde_json::Value>> {
    let capabilities = ctx.client.capabilities(ctx.clone()).await?;

    let tools = capabilities
        .rest
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.resource.unwrap_or_default())
        .flatten()
        .map(|resource_rest_capability| {
            let resource_name: Option<String> = resource_rest_capability.type_.as_ref().into();
            let resource_name = resource_name.unwrap_or_default();

            Tool {
                annotations: None,
                description: Some(format!(
                    "Tool operation for FHIR Resource '{}'",
                    resource_name
                )),
                execution: None,
                icons: vec![],
                input_schema: generate_resource_schema(&resource_rest_capability),
                meta: None,
                name: resource_name.clone(),
                output_schema: None,
                title: Some(format!("FHIR {} Operations Tool", resource_name)),
            }
        })
        .collect::<Vec<_>>();

    Ok(ListToolsResult {
        tools: tools,
        meta: None,
        next_cursor: None,
    })
}
