use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

use crate::elastic_search::search::QueryBuildError;

pub fn uri(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let uri_params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            Ok(json!({
                "match":{
                    search_param.url.value.as_ref().unwrap(): {
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
