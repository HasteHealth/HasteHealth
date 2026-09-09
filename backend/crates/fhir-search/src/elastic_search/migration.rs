use elasticsearch::{
    Elasticsearch,
    indices::{
        IndicesCreateParts, IndicesDeleteParts, IndicesGetMappingParts, IndicesGetSettingsParts,
        IndicesPutMappingParts, IndicesPutSettingsParts,
    },
    params::Slices,
};
use haste_fhir_model::r4::generated::terminology::SearchParamType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use serde_json::{Map, Value, json};
use std::{collections::HashMap, sync::Arc};

use crate::{
    ResolvedParameter, SearchParameterResolve,
    elastic_search::{DYNAMIC_PARAMETER_INDEX_FIELD, flatten_parameter_field_name},
};

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

pub fn create_elasticsearch_searchparameter_mappings(parameters: &[ResolvedParameter]) -> Value {
    let mut property_mapping: HashMap<String, Value> = HashMap::new();
    for parameter in parameters {
        let search_parameter = &parameter.search_parameter;
        if let Some(parameter_url) = search_parameter.url.value.as_ref() {
            let field_name = flatten_parameter_field_name(parameter_url);
            match &search_parameter.type_ {
                param_type if param_type == &SearchParamType::number() => {
                    property_mapping.insert(field_name, number_index_mapping());
                }
                param_type if param_type == &SearchParamType::string() => {
                    property_mapping.insert(field_name, string_index_mapping());
                }
                param_type if param_type == &SearchParamType::uri() => {
                    property_mapping.insert(field_name, uri_index_mapping());
                }
                param_type if param_type == &SearchParamType::token() => {
                    property_mapping.insert(field_name, token_index_mapping());
                }
                param_type if param_type == &SearchParamType::date() => {
                    property_mapping.insert(field_name, date_index_mapping());
                }
                param_type if param_type == &SearchParamType::reference() => {
                    property_mapping.insert(field_name, reference_index_mapping());
                }
                param_type if param_type == &SearchParamType::quantity() => {
                    property_mapping.insert(field_name, quantity_index_mapping());
                }
                // Not Supported yet
                param_type
                    if param_type == &SearchParamType::composite()
                        || param_type == &SearchParamType::special()
                        || param_type == &SearchParamType::null() =>
                {
                    tracing::warn!("Unsupported search parameter type");
                }
                _ => {
                    tracing::warn!("Unsupported search parameter type");
                }
            }
        }
    }

    property_mapping.insert(
        DYNAMIC_PARAMETER_INDEX_FIELD.to_string(),
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

    // Safe because every document field name comes from one of two sources:
    // System-level parameters, flattened into their own top-level field below
    // (and enumerated here via `ParameterLevel::System`, which only
    // `R4_SEARCH_PARAMETERS_INDEX` ever produces), or Project-level (tenant
    // custom) parameters, which always route through the single
    // `dynamic_parameters` nested field mapped below rather than getting a
    // field name of their own. No write can introduce an unmapped key.
    json!({
        "dynamic": "strict",
        "properties" : property_mapping
    })
}

/// Compares the property keys of an expected mapping against an index's
/// current mapping. Pure and index-agnostic so it can be unit tested without
/// a live Elasticsearch cluster.
fn diff_property_keys(
    expected: &Map<String, Value>,
    current: &Map<String, Value>,
) -> (Vec<String>, Vec<String>) {
    let mut added: Vec<String> = expected
        .keys()
        .filter(|key| !current.contains_key(*key))
        .cloned()
        .collect();
    let mut removed: Vec<String> = current
        .keys()
        .filter(|key| !expected.contains_key(*key))
        .cloned()
        .collect();

    added.sort();
    removed.sort();

    (added, removed)
}

/// Constructs the body for creating an Elasticsearch index, including settings
/// for nested and total fields limits, and optionally the number of replicas.
fn index_creation_body(mapping_body: &Value, number_of_replicas: Option<u32>) -> Value {
    let mut index_settings = json!({
        "mapping": {
            "nested_fields": {
                "limit": 2000
            },
            "total_fields": {
                "limit": 10000
            }
        }
    });

    if let Some(replicas) = number_of_replicas {
        index_settings["number_of_replicas"] = json!(replicas);
    }

    json!({
       "settings": {
           "index": index_settings
       },
       "mappings": mapping_body
    })
}

async fn create_index(
    elastic_search: &Elasticsearch,
    index: &str,
    mapping_body: &Value,
    number_of_replicas: Option<u32>,
) {
    let res = elastic_search
        .indices()
        .create(IndicesCreateParts::Index(index))
        .body(index_creation_body(mapping_body, number_of_replicas))
        .send()
        .await
        .unwrap();

    if res.status_code().is_success() {
        tracing::info!("Elasticsearch index '{}' created successfully.", index);
    } else {
        tracing::error!(
            "Failed to create Elasticsearch index '{}': {:?}",
            index,
            res
        );
        tracing::error!("Response: {:?}", res.text().await.unwrap());
        panic!();
    }
}

async fn delete_index(elastic_search: &Elasticsearch, index: &str) {
    let res = elastic_search
        .indices()
        .delete(IndicesDeleteParts::Index(&[index]))
        .send()
        .await
        .unwrap();

    if res.status_code().is_success() {
        tracing::info!("Elasticsearch index '{}' deleted successfully.", index);
    } else {
        tracing::error!(
            "Failed to delete Elasticsearch index '{}': {:?}",
            index,
            res
        );
        tracing::error!("Response: {:?}", res.text().await.unwrap());
        panic!();
    }
}

/// Copies documents from `from_index` into `to_index`. When `keep_fields` is
/// set, only those top-level `_source` fields are carried over -- this is
/// what actually drops data for search parameters that no longer exist,
/// since Elasticsearch has no API to remove a field from a mapping in place.
async fn reindex(
    elastic_search: &Elasticsearch,
    from_index: &str,
    to_index: &str,
    keep_fields: Option<&[String]>,
) {
    let mut source = json!({ "index": from_index });
    if let Some(fields) = keep_fields {
        source["_source"] = json!(fields);
    }

    let res = elastic_search
        .reindex()
        .body(json!({
            "source": source,
            "dest": { "index": to_index }
        }))
        .refresh(true)
        .slices(Slices::Auto)
        .wait_for_completion(true)
        .send()
        .await
        .unwrap();

    if res.status_code().is_success() {
        tracing::info!(
            "Reindexed '{}' into '{}' successfully.",
            from_index,
            to_index
        );
    } else {
        tracing::error!(
            "Failed to reindex '{}' into '{}': {:?}",
            from_index,
            to_index,
            res
        );
        tracing::error!("Response: {:?}", res.text().await.unwrap());
        panic!();
    }
}

/// Reads the index's current `number_of_replicas` so a bulk rebuild can
/// restore it afterward instead of leaving the index under-replicated.
/// Falls back to `1` (Elasticsearch's own default) if the setting can't be
/// read, since that's a safer failure mode than leaving replicas at 0.
async fn fetch_replica_count(elastic_search: &Elasticsearch, index: &str) -> u32 {
    let res = elastic_search
        .indices()
        .get_settings(IndicesGetSettingsParts::Index(&[index]))
        .send()
        .await
        .unwrap();

    if !res.status_code().is_success() {
        tracing::error!(
            "Failed to fetch Elasticsearch settings for index '{}': {:?}",
            index,
            res
        );
        return 1;
    }

    let body: Value = res.json().await.unwrap();

    body.get(index)
        .and_then(|v| v.get("settings"))
        .and_then(|v| v.get("index"))
        .and_then(|v| v.get("number_of_replicas"))
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1)
}

async fn set_replica_count(elastic_search: &Elasticsearch, index: &str, number_of_replicas: u32) {
    let res = elastic_search
        .indices()
        .put_settings(IndicesPutSettingsParts::Index(&[index]))
        .body(json!({ "index": { "number_of_replicas": number_of_replicas } }))
        .send()
        .await
        .unwrap();

    if res.status_code().is_success() {
        tracing::info!(
            "Restored 'number_of_replicas' to {} on index '{}'.",
            number_of_replicas,
            index
        );
    } else {
        tracing::error!(
            "Failed to restore 'number_of_replicas' on index '{}': {:?}",
            index,
            res
        );
        tracing::error!("Response: {:?}", res.text().await.unwrap());
        panic!();
    }
}

struct CurrentMapping {
    properties: Map<String, Value>,
    dynamic_is_strict: bool,
}

async fn fetch_current_mapping(elastic_search: &Elasticsearch, index: &str) -> CurrentMapping {
    let res = elastic_search
        .indices()
        .get_mapping(IndicesGetMappingParts::Index(&[index]))
        .send()
        .await
        .unwrap();

    if !res.status_code().is_success() {
        tracing::error!(
            "Failed to fetch Elasticsearch mapping for index '{}': {:?}",
            index,
            res
        );
        panic!();
    }

    let body: Value = res.json().await.unwrap();
    let mappings = body.get(index).and_then(|v| v.get("mappings"));

    let properties = mappings
        .and_then(|v| v.get("properties"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let dynamic_is_strict = mappings
        .and_then(|v| v.get("dynamic"))
        .and_then(Value::as_str)
        == Some("strict");

    CurrentMapping {
        properties,
        dynamic_is_strict,
    }
}

/// Merges newly-added search parameter columns into an existing index's
/// mapping. Always safe to run in place -- Elasticsearch mappings only ever
/// grow via `put_mapping`, so this never touches already-mapped fields.
async fn apply_additive_mapping(elastic_search: &Elasticsearch, index: &str, mapping_body: &Value) {
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
}

/// Brings an index's mapping in sync with the current set of search
/// parameters: new parameters get their column added, and parameters that no
/// longer exist have their column (and indexed data) dropped.
///
/// Adding columns is a plain `put_mapping` merge, always applied. Removing
/// columns isn't -- Elasticsearch mappings are append-only -- so dropping one
/// requires rebuilding the index from scratch under a staging name, keeping
/// only the fields that are still expected, then swapping it back into place
/// under the original index name. That rebuild briefly makes the index
/// unavailable, so it only runs when `prune_removed_parameters` is `true`;
/// otherwise removed parameters are just logged.
pub async fn create_mapping<ParameterResolver: SearchParameterResolve>(
    parameter_resolver: Arc<ParameterResolver>,
    elastic_search: &Elasticsearch,
    index: &str,
    prune_removed_parameters: bool,
) -> Result<(), OperationOutcomeError> {
    let exists_res = elastic_search
        .indices()
        .exists(elasticsearch::indices::IndicesExistsParts::Index(&[index]))
        .send()
        .await
        .unwrap();

    let expected_mapping_body = create_elasticsearch_searchparameter_mappings(
        &parameter_resolver
            .all(&TenantId::System, &ProjectId::System)
            .await?,
    );

    let index_exists = exists_res.status_code().is_success();

    if !index_exists {
        create_index(elastic_search, index, &expected_mapping_body, None).await;
        return Ok(());
    }

    let expected_properties = expected_mapping_body
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let current_mapping = fetch_current_mapping(elastic_search, index).await;

    let (added, removed) = diff_property_keys(&expected_properties, &current_mapping.properties);

    if added.is_empty() && removed.is_empty() {
        if !current_mapping.dynamic_is_strict {
            tracing::info!(
                "Elasticsearch mapping for index '{}' has matching columns but is not yet 'dynamic: strict' -- updating.",
                index
            );
            apply_additive_mapping(elastic_search, index, &expected_mapping_body).await;
            return Ok(());
        }

        tracing::info!(
            "Elasticsearch mapping for index '{}' already in sync with search parameters.",
            index
        );
        return Ok(());
    }

    if !removed.is_empty() && !prune_removed_parameters {
        tracing::warn!(
            "{} search parameter column(s) on index '{}' are no longer backed by an active SearchParameter and would be dropped: {:?}. Not rebuilding the index because `prune_removed_search_parameters` is disabled.",
            removed.len(),
            index,
            removed
        );

        if !added.is_empty() {
            tracing::info!(
                "Adding {} new search parameter column(s) to index '{}': {:?}",
                added.len(),
                index,
                added
            );
            apply_additive_mapping(elastic_search, index, &expected_mapping_body).await;
        }

        return Ok(());
    }

    if removed.is_empty() {
        tracing::info!(
            "Adding {} new search parameter column(s) to index '{}': {:?}",
            added.len(),
            index,
            added
        );
        apply_additive_mapping(elastic_search, index, &expected_mapping_body).await;
        return Ok(());
    }

    tracing::info!(
        "Search parameter columns changed for index '{}' -- {} added {:?}, {} removed {:?}. Rebuilding index.",
        index,
        added.len(),
        added,
        removed.len(),
        removed
    );

    let expected_keys: Vec<String> = expected_properties.keys().cloned().collect();
    let staging_index = format!("{}_migrate_{}", index, chrono::Utc::now().timestamp());

    // Replicas are dropped to 0 for both rebuild copies neither is
    // read-facing until the final swap below and restored on the
    // rebuilt `index` afterward so it doesn't stay under-replicated.
    let original_replicas = fetch_replica_count(elastic_search, index).await;

    // create Staging Index
    create_index(
        elastic_search,
        &staging_index,
        &expected_mapping_body,
        Some(0),
    )
    .await;
    reindex(elastic_search, index, &staging_index, Some(&expected_keys)).await;

    // Remove old index and create new one with updated mapping, then reindex from staging
    delete_index(elastic_search, index).await;
    create_index(elastic_search, index, &expected_mapping_body, Some(0)).await;
    reindex(elastic_search, &staging_index, index, None).await;

    // Restore the original replica count and delete the staging index
    set_replica_count(elastic_search, index, original_replicas).await;
    delete_index(elastic_search, &staging_index).await;

    tracing::info!(
        "Elasticsearch index '{}' rebuilt successfully with synced mapping.",
        index
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(keys: &[&str]) -> Map<String, Value> {
        keys.iter()
            .map(|key| ((*key).to_string(), json!({ "type": "keyword" })))
            .collect()
    }

    #[test]
    fn diff_property_keys_detects_additions_and_removals() {
        let expected = props(&["patient", "status", "code"]);
        let current = props(&["patient", "status", "old-param"]);

        let (added, removed) = diff_property_keys(&expected, &current);

        assert_eq!(added, vec!["code".to_string()]);
        assert_eq!(removed, vec!["old-param".to_string()]);
    }

    #[test]
    fn diff_property_keys_empty_when_in_sync() {
        let expected = props(&["patient", "status"]);
        let current = props(&["patient", "status"]);

        let (added, removed) = diff_property_keys(&expected, &current);

        assert!(added.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn diff_property_keys_handles_pure_addition() {
        let expected = props(&["patient", "status"]);
        let current = props(&["patient"]);

        let (added, removed) = diff_property_keys(&expected, &current);

        assert_eq!(added, vec!["status".to_string()]);
        assert!(removed.is_empty());
    }
}
