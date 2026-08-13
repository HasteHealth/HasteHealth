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
        .map(|value| build_date_query(value, &column_name))
        .collect::<Result<Vec<_>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}

fn build_date_query(value: &str, column_name: &str) -> Result<serde_json::Value, QueryBuildError> {
    let (prefix, value) = parse_prefix(value);

    let date_time = parse_datetime(value)
        .map_err(|_e| QueryBuildError::InvalidDateFormat(value.to_string()))?;

    let date_range = date_time_range(&date_time)
        .map_err(|_e| QueryBuildError::InvalidDateFormat(value.to_string()))?;

    match prefix {
        Some("gt") => Ok(date_range_query(
            column_name,
            &json!({
                "gt": date_range.end
            }),
        )),

        Some("lt") => Ok(date_range_query(
            column_name,
            &json!({
                "lt": date_range.start
            }),
        )),

        Some("ge") => Ok(date_range_query(
            column_name,
            &json!({
                "gte": date_range.start
            }),
        )),

        Some("le") => Ok(date_range_query(
            column_name,
            &json!({
                "lte": date_range.end
            }),
        )),

        Some("ne") => Ok(date_not_overlapping_query(
            column_name,
            date_range.start,
            date_range.end,
        )),

        Some("eq") | None => Ok(date_overlapping_query(
            column_name,
            date_range.start,
            date_range.end,
        )),

        Some(prefix) => Err(QueryBuildError::UnsupportedModifier(prefix.to_string())),
    }
}

fn date_range_query(column_name: &str, range: &serde_json::Value) -> serde_json::Value {
    json!({
        "nested": {
            "path": column_name,
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                format!("{}.start", column_name): range
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn date_overlapping_query(
    column_name: &str,
    start: impl serde::Serialize,
    end: impl serde::Serialize,
) -> serde_json::Value {
    json!({
        "nested": {
            "path": column_name,
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                format!("{}.start", column_name): {
                                    "lte": end
                                }
                            }
                        },
                        {
                            "range": {
                                format!("{}.end", column_name): {
                                    "gte": start
                                }
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn date_not_overlapping_query(
    column_name: &str,
    start: impl serde::Serialize,
    end: impl serde::Serialize,
) -> serde_json::Value {
    json!({
        "nested": {
            "path": column_name,
            "query": {
                "bool": {
                    "must_not": [
                        {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            format!("{}.start", column_name): {
                                                "lte": end
                                            }
                                        }
                                    },
                                    {
                                        "range": {
                                            format!("{}.end", column_name): {
                                                "gte": start
                                            }
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        }
    })
}
