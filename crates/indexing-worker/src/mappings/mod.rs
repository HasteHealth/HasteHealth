use oxidized_fhir_model::r4::types::SearchParameter;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde_json::{Value, json};
use std::collections::HashMap;

// Note use of nested because must preserve groupings of fields.
pub fn date_index_mapping() -> serde_json::Value {
    json!({
        "type": "nested",
        "properties": {
            "start": { "type": "date" },
            "end": { "type": "date" },
        }
    })
}

pub fn string_index_mapping() -> serde_json::Value {
    json!({
        "type": "text"
    })
}

pub fn token_index_mapping() -> serde_json::Value {
    json!({
        "type": "nested",
        "properties": {
            "system": { "type": "keyword" },
            "code": { "type": "keyword" },
            "display": { "type": "text" }
        }
    })
}

pub fn number_index_mapping() -> serde_json::Value {
    json!({
        "type": "long"
    })
}

pub fn uri_index_mapping() -> serde_json::Value {
    json!({
        "type": "keyword"
    })
}

pub fn quantity_index_mapping() -> serde_json::Value {
    json!({

        "type": "nested",
        "properties": {
            "low": { "type": "long" },
            "high": { "type": "long" }
        }

    })
}

pub fn reference_index_mapping() -> serde_json::Value {
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
    search_parameters: &Vec<&SearchParameter>,
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

    Ok(json!({
        "properties" : property_mapping
    }))
}
