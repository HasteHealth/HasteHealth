use elasticsearch::{
    Elasticsearch,
    indices::{IndicesCreateParts, IndicesPutMappingParts},
};
use haste_fhir_model::r4::generated::terminology::SearchParamType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

use crate::{ResolvedParameter, SearchParameterResolve};

// Note use of nested because must preserve groupings of fields.
fn date_index_mapping() -> serde_json::Value {
    json!({
        "type": "nested",
        "properties": {
            "start": { "type": "date" },
            "end": { "type": "date" },
        }
    })
}

fn string_index_mapping() -> serde_json::Value {
    json!({
        "type": "keyword"
    })
}

fn token_index_mapping() -> serde_json::Value {
    json!({
        "type": "nested",
        "properties": {
            "system": { "type": "keyword" },
            "code": { "type": "keyword" },
            "display": { "type": "keyword" }
        }
    })
}

fn number_index_mapping() -> serde_json::Value {
    json!({
        "type": "double"
    })
}

fn uri_index_mapping() -> serde_json::Value {
    json!({
        "type": "keyword"
    })
}

fn quantity_index_mapping() -> serde_json::Value {
    json!({
        "type": "nested",
        "properties": {
            "start_value": { "type": "double" },
            "start_system": { "type": "keyword" },
            "start_code": { "type": "keyword" },

            "end_value": { "type": "double" },
            "end_system": { "type": "keyword" },
            "end_code": { "type": "keyword" }
        }

    })
}

fn reference_index_mapping() -> serde_json::Value {
    json!({
        "type": "nested",
        "properties": {
            "resource_type": { "type": "keyword" },
            "id": { "type": "keyword" },
            "uri": { "type": "keyword" }
        }

    })
}

pub async fn create_elasticsearch_searchparameter_mappings(
    parameters: &Vec<ResolvedParameter>,
) -> Result<Value, OperationOutcomeError> {
    let mut property_mapping: HashMap<String, Value> = HashMap::new();
    for parameter in parameters.iter() {
        let search_parameter = &parameter.search_parameter;
        if let Some(parameter_url) = search_parameter.url.value.as_ref() {
            match search_parameter.type_.as_ref() {
                SearchParamType::Number(_) => {
                    property_mapping.insert(parameter_url.to_string(), number_index_mapping());
                }
                SearchParamType::String(_) => {
                    property_mapping.insert(parameter_url.to_string(), string_index_mapping());
                }
                SearchParamType::Uri(_) => {
                    property_mapping.insert(parameter_url.to_string(), uri_index_mapping());
                }
                SearchParamType::Token(_) => {
                    property_mapping.insert(parameter_url.to_string(), token_index_mapping());
                }
                SearchParamType::Date(_) => {
                    property_mapping.insert(parameter_url.to_string(), date_index_mapping());
                }
                SearchParamType::Reference(_) => {
                    property_mapping.insert(parameter_url.to_string(), reference_index_mapping());
                }
                SearchParamType::Quantity(_) => {
                    property_mapping.insert(parameter_url.to_string(), quantity_index_mapping());
                }
                // Not Supported yet
                SearchParamType::Composite(_)
                | SearchParamType::Special(_)
                | SearchParamType::Null(_) => {
                    tracing::warn!("Unsupported search parameter type");
                }
            }
        }
    }

    property_mapping.insert(
        "dynamic_parameters".to_string(),
        json!({
            "type": "nested",
            "properties": {
                "url": { "type": "keyword" },
                "type": { "type": "keyword" },
                "value": {
                    "type": "object",
                    "properties": {
                        "string": string_index_mapping(),
                        "number": number_index_mapping(),
                        "date": date_index_mapping(),
                        "uri": uri_index_mapping(),
                        "token": token_index_mapping(),
                        "quantity": quantity_index_mapping(),
                        "reference": reference_index_mapping()
                    }
                }
            }
        }),
    );

    property_mapping.insert(
        "resource_type".to_string(),
        json!({
            "type": "keyword",
        }),
    );

    property_mapping.insert(
        "id".to_string(),
        json!({
            "index": false,
            "type": "keyword"
        }),
    );

    property_mapping.insert(
        "version_id".to_string(),
        json!({
            "index": false,
            "type": "keyword"
        }),
    );

    property_mapping.insert(
        "tenant".to_string(),
        json!({
            "type": "keyword",
        }),
    );

    property_mapping.insert(
        "project".to_string(),
        json!({
            "type": "keyword",
        }),
    );

    Ok(json!({
        "properties" : property_mapping
    }))
}

pub async fn create_mapping<ParameterResolver: SearchParameterResolve>(
    parameter_resolver: Arc<ParameterResolver>,
    elastic_search: &Elasticsearch,
    index: &str,
) -> Result<(), OperationOutcomeError> {
    let exists_res = elastic_search
        .indices()
        .exists(elasticsearch::indices::IndicesExistsParts::Index(&vec![
            index,
        ]))
        .send()
        .await
        .unwrap();

    let mapping_body = create_elasticsearch_searchparameter_mappings(
        &parameter_resolver
            .all(&TenantId::System, &ProjectId::System)
            .await?,
    )
    .await
    .unwrap();

    let index_exists = exists_res.status_code().is_success();

    if index_exists {
        let res = elastic_search
            .indices()
            .put_mapping(IndicesPutMappingParts::Index(&[index]))
            .body(mapping_body)
            .send()
            .await
            .unwrap();
        if res.status_code().is_success() {
            tracing::info!("Elasticsearch mapping updated successfully.");
        } else {
            tracing::error!("Failed to update Elasticsearch mapping: {:?}", res);
            tracing::error!("Response: {:?}", res.text().await.unwrap());
            panic!();
        }
    } else {
        let res = elastic_search
            .indices()
            .create(IndicesCreateParts::Index(index))
            .body(json!({
                   "settings": {
                       "index": {
                            "mapping": {
                                "nested_fields": {
                                    "limit": 2000
                                },
                                "total_fields": {
                                    "limit": 5000
                                }
                            }
                       }
                   },
                   "mappings": mapping_body
            }))
            .send()
            .await
            .unwrap();

        if res.status_code().is_success() {
            tracing::info!("Elasticsearch mapping created successfully.");
        } else {
            tracing::error!("Failed to create Elasticsearch mapping: {:?}", res);
            tracing::error!("Response: {:?}", res.text().await.unwrap());
            panic!();
        }
    }

    Ok(())
}
