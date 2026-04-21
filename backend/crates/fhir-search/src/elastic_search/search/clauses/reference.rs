use crate::elastic_search::search::{QueryBuildError, clauses::namespace_parameter};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn reference(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);

    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let pieces = value.split('/').collect::<Vec<&str>>();
            match pieces.len() {
                1 => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "match": {
                                format!("{}.id", &column_name): {
                                    "query": pieces.get(0)
                                }
                            }
                        }
                    }
                })),
                2 => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "match": {
                                            format!("{}.resource_type", &column_name): {
                                                "query": pieces.get(0)
                                            }
                                        }
                                    },
                                    {
                                        "match": {
                                            format!("{}.id", &column_name): {
                                                "query": pieces.get(1)
                                            }
                                        }
                                    }
                                ]
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
