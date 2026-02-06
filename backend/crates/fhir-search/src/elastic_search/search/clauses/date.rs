use crate::{elastic_search::search::QueryBuildError, indexing_conversion::date_time_range};
use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::{datetime::parse_datetime, generated::resources::SearchParameter};
use serde_json::json;

pub fn date(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let (prefix, value) = parse_prefix(value);

            let date_time = parse_datetime(value)
                .map_err(|_e| QueryBuildError::InvalidDateFormat(value.to_string()))?;
            let date_range = date_time_range(&date_time)
                .map_err(|_e| QueryBuildError::InvalidDateFormat(value.to_string()))?;
            let search_param_url = search_param
                .url
                .value
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default()
                .to_string();

            match prefix {
                Some("gt") => Ok(json!({
                    "nested": {
                        "path": search_param_url,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            search_param_url + ".start": {
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
                        "path": search_param_url,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            search_param_url + ".end": {
                                                "lt": date_range.start
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
                        "path": search_param_url,
                        "query": {
                            "bool": {
                                "filter": [
                                    {
                                        "range": {
                                            search_param_url.clone() + ".start": {
                                                "gte": date_range.start
                                            }
                                        }
                                    },
                                    {
                                        "range": {
                                            search_param_url + ".end": {
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
