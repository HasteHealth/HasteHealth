use elasticsearch::{Elasticsearch, indices::IndicesCreateParts};
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

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
    search_parameters: &Vec<Arc<oxidized_fhir_model::r4::types::SearchParameter>>,
) -> Result<Value, OperationOutcomeError> {
    let mut property_mapping: HashMap<String, Value> = HashMap::new();
    for parameter in search_parameters.iter() {
        if let Some(parameter_url) = parameter.url.value.as_ref() {
            match parameter.type_.value.as_ref().map(|v| v.as_str()) {
                Some("number") => {
                    property_mapping.insert(parameter_url.to_string(), number_index_mapping());
                }
                Some("string") => {
                    property_mapping.insert(parameter_url.to_string(), string_index_mapping());
                }
                Some("uri") => {
                    property_mapping.insert(parameter_url.to_string(), uri_index_mapping());
                }
                Some("token") => {
                    property_mapping.insert(parameter_url.to_string(), token_index_mapping());
                }
                Some("date") => {
                    property_mapping.insert(parameter_url.to_string(), date_index_mapping());
                }
                Some("reference") => {
                    property_mapping.insert(parameter_url.to_string(), reference_index_mapping());
                }
                Some("quantity") => {
                    property_mapping.insert(parameter_url.to_string(), quantity_index_mapping());
                }
                // Not Supported yet
                Some("composite") | Some(_) | None => {
                    tracing::warn!("Unsupported search parameter type");
                }
            }
        }
    }

    property_mapping.insert(
        "resource_type".to_string(),
        json!({
            "type": "keyword",
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

pub async fn create_mapping(
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

    if !exists_res.status_code().is_success() {
        let mapping_body = create_elasticsearch_searchparameter_mappings(
            &oxidized_artifacts::search_parameters::get_all_search_parameters(),
        )
        .await
        .unwrap();
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
