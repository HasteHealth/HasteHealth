use crate::elastic_search::search::{QueryBuildError, clauses::namespace_parameter};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

fn matching_modifier(modifier: &Option<String>) -> Result<String, QueryBuildError> {
    match modifier.as_ref().map(|s| s.as_str()) {
        Some("not") => Ok("must_not".to_string()),
        Some(modifier) => Err(QueryBuildError::ModifierNotSupported(modifier.into())),
        None => Ok("must".to_string()),
    }
}

pub fn token(
    namespace: Option<&str>,
    parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let matching_type = matching_modifier(&parameter.modifier)?;
    let column_name = namespace_parameter(namespace, search_param);

    let params = parameter
        .value
        .iter()
        .map(|value| {
            let pieces = value.split('|').collect::<Vec<&str>>();
            match pieces.len() {
                1 => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                &matching_type: [{
                                    "match": {
                                        format!("{}.code", column_name): {
                                        "query": pieces.get(0)
                                    }
                                }}]
                            }
                        }
                    }
                })),
                2 => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                &matching_type: [{
                                    "bool": {
                                        "filter": [
                                            {
                                                "match": {
                                                    format!("{}.code", column_name): {
                                                        "query": pieces.get(1)
                                                    }
                                                }
                                            },
                                            {
                                                "match": {
                                                    format!("{}.system", column_name): {
                                                        "query": pieces.get(0)
                                                    }
                                                }
                                            }
                                        ]
                                    }
                                }]
                            }
                        }
                    }
                })),
                _ => Err(QueryBuildError::InvalidParameterValue(value.to_string())),
            }
        })
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}
