use crate::{
    elastic_search::search::{QueryBuildError, simple_missing_modifier},
    indexing_conversion::get_decimal_range,
};
use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn number(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    match parsed_parameter.modifier.as_ref().map(|m| m.as_str()) {
        Some("missing") => {
            return simple_missing_modifier(search_param, parsed_parameter);
        }
        Some(modifier) => {
            return Err(QueryBuildError::UnsupportedModifier(modifier.to_string()));
        }
        None => {
            let params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    let (prefix, value) = parse_prefix(value);
                    let range = get_decimal_range(value)
                        .map_err(|_e| QueryBuildError::InvalidParameterValue(value.to_string()))?;

                    match prefix {
                        Some("ne") => Ok(json!({
                            "bool": {
                                "must_not": {
                                    "range": {
                                        search_param.url.value.as_ref().unwrap(): {
                                            "gte": range.start,
                                            "lte": range.end
                                        }
                                    }
                                }
                            }
                        })),
                        Some("gt") => Ok(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap(): {
                                    "gt": range.end
                                }
                            }
                        })),
                        Some("lt") => Ok(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap(): {
                                    "lt": range.start
                                }
                            }
                        })),
                        Some("ge") => Ok(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap(): {
                                    "gte": range.start
                                }
                            }
                        })),
                        Some("le") => Ok(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap(): {
                                    "lte": range.end
                                }
                            }
                        })),
                        Some("eq") | None => Ok(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap(): {
                                    "gte": range.start,
                                    "lte": range.end
                                }
                            }
                        })),
                        Some(prefix) => Err(QueryBuildError::UnsupportedPrefix(prefix.to_string())),
                    }
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": params
                }
            }))
        }
    }
}
