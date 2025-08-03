use elasticsearch::{Elasticsearch, indices::IndicesPutMappingParts};
use oxidized_fhir_model::r4::types::SearchParameter;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde_json::json;

use crate::R4_FHIR_INDEX;

pub fn date_index_mapping(url: &str) -> serde_json::Value {
    json!({
        url: {
            "type": "nested",
            "properties": {
                "start": { "type": "date" },
                "end": { "type": "date" },
            }
        }
    })
}

pub fn string_index_mapping(url: &str) -> serde_json::Value {
    json!({
        url: {
            "type": "text",
        }
    })
}

pub fn token_index_mapping(url: &str) -> serde_json::Value {
    json!({
        url: {
            "type": "keyword",
        }
    })
}

pub fn number_index_mapping(url: &str) -> serde_json::Value {
    json!({
        url: {
            "type": "long",
        }
    })
}

pub fn quantity_index_mapping(url: &str) -> serde_json::Value {
    json!({
        url: {
            "type": "nested",
            "properties": {
                "low": { "type": "long" },
                "high": { "type": "long" },
            }
        }
    })
}

pub fn reference_index_mapping(url: &str) -> serde_json::Value {
    json!({
        url: {
            "type": "nested",
            "properties": {
                "resource_type": { "type": "keyword" },
                "id": { "type": "keyword" },
                "uri": { "type": "keyword" },
            }
        }
    })
}

pub async fn create_mappings(
    client: &Elasticsearch,
    search_parameters: &Vec<&SearchParameter>,
) -> Result<(), OperationOutcomeError> {
    client
        .indices()
        .put_mapping(IndicesPutMappingParts::Index(&[R4_FHIR_INDEX]))
        .body(json!({
            "properties" : {
                "field1" : { "type" : "text" }
            }
        }))
        .send()
        .await
        .unwrap();

    Ok(())
}
