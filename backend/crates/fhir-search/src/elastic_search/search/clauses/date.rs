use crate::{
    elastic_search::search::{QueryBuildError, clauses::namespace_parameter},
    indexing_conversion::date_time_range,
};
use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::{datetime::parse_datetime, generated::resources::SearchParameter};
use serde_json::json;

pub fn date(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let (prefix, value) = parse_prefix(value);
            let date_time = parse_datetime(value)
                .map_err(|_e| QueryBuildError::InvalidDateFormat(value.to_string()))?;
            let date_range = date_time_range(&date_time)
                .map_err(|_e| QueryBuildError::InvalidDateFormat(value.to_string()))?;

            match prefix {
                Some("gt") => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            format!("{}.start", column_name): {
                                                "gt": date_range.end
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    }
                })),
                Some("lt") => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            format!("{}.end", column_name): {
                                                "lt": date_range.start
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    }
                })),
                Some("ge") => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            format!("{}.start", column_name): {
                                                "gte": date_range.start
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    }
                })),
                Some("le") => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            format!("{}.end", column_name): {
                                                "lte": date_range.end
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    }
                })),
                Some("eq") | None => Ok(json!({
                    "nested": {
                        "path": &column_name,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            format!("{}.start", column_name): {
                                                "gte": date_range.start
                                            }
                                        }
                                    },
                                    {
                                        "range": {
                                            format!("{}.end", column_name): {
                                                "lte": date_range.end
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    }
                })),
                Some(prefix) => Err(QueryBuildError::UnsupportedModifier(prefix.to_string())),
            }
        })
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}
