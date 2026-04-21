use crate::{
    elastic_search::search::{
        QueryBuildError, clauses::namespace_parameter, simple_missing_modifier,
    },
    indexing_conversion::get_decimal_range,
};
use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn number(
    namespace: Option<&str>,
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
            let column_name = namespace_parameter(namespace, search_param);
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
                                        &column_name: {
                                            "gte": range.start,
                                            "lte": range.end
                                        }
                                    }
                                }
                            }
                        })),
                        Some("gt") => Ok(json!({
                            "range": {
                                &column_name: {
                                    "gt": range.end
                                }
                            }
                        })),
                        Some("lt") => Ok(json!({
                            "range": {
                                &column_name: {
                                    "lt": range.start
                                }
                            }
                        })),
                        Some("ge") => Ok(json!({
                            "range": {
                                &column_name: {
                                    "gte": range.start
                                }
                            }
                        })),
                        Some("le") => Ok(json!({
                            "range": {
                                &column_name: {
                                    "lte": range.end
                                }
                            }
                        })),
                        Some("eq") | None => Ok(json!({
                            "range": {
                                &column_name: {
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
