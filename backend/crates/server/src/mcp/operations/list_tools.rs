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
    resources::{CapabilityStatement, CapabilityStatementRestResourceSearchParam},
    terminology::SearchParamType,
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

fn generate_search_schema(capabilities: &CapabilityStatement) -> Tool {
    let default_ = vec![];
    let resource_capabilities = capabilities
        .rest
        .as_ref()
        .unwrap_or(&default_)
        .into_iter()
        .filter_map(|r| r.resource.as_ref())
        .flatten()
        .collect::<Vec<_>>();

    let input_schema = json!({
      "type": "object",
      "properties": {
        "resourceType": {
          "type": "string",
          "enum": resource_capabilities.iter().map(|rc| {
              let resource_type: Option<String> = rc.type_.as_ref().into();
              resource_type.unwrap_or_default()
          }).collect::<Vec<String>>(),
        }
      },
      "required": ["resourceType"],
      "oneOf" : resource_capabilities.iter().map(|rc| {
          let resource_type: Option<String> = rc.type_.as_ref().into();
          let resource_type = resource_type.unwrap_or_default();
          json!({
              "if": {
                  "properties": {
                      "resourceType": { "const": resource_type }
                  }
              },
              "then": {
                  "properties": {
                      "search_parameters": {
                          "type": "object",
                          "properties": search_tool_parameters(
                              rc.searchParam.as_ref().unwrap_or(&vec![])
                          ),
                      }
                  }
              }
          })
      }).collect::<Vec<serde_json::Value>>()
    });

    Tool {
        annotations: None,
        description: Some(format!(
            "Tool for FHIR Resource Search across supported types",
        )),
        execution: None,
        icons: vec![],
        input_schema,
        meta: None,
        name: "fhir_r4_search".to_string(),
        output_schema: Some(haste_sd_to_json_schema::bundle_of_resource(json!({
            "type": "object"
        }))),
        title: Some("fhir_r4_search".to_string()),
    }
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
    let search_tool = generate_search_schema(&capabilities);

    Ok(ListToolsResult {
        tools: vec![search_tool],
        meta: None,
        next_cursor: None,
    })
}
