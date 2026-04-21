use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

use crate::elastic_search::search::{
    QueryBuildError, clauses::namespace_parameter, simple_missing_modifier,
};

pub fn uri(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_parameter: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    match parsed_parameter.modifier.as_ref().map(|m| m.as_str()) {
        Some("missing") => {
            return simple_missing_modifier(search_parameter, parsed_parameter);
        }
        Some(modifier) => {
            return Err(QueryBuildError::UnsupportedModifier(modifier.to_string()));
        }
        None => {
            let column_name = namespace_parameter(namespace, search_parameter);

            let uri_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "match":{
                            &column_name: {
                                "query": value
                            }
                        }
                    }))
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": uri_params
                }
            }))
        }
    }
}
