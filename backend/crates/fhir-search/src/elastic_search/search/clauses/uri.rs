use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

use crate::elastic_search::search::clauses::namespace_parameter;

pub fn uri(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_parameter: &SearchParameter,
) -> serde_json::Value {
    let column_name = namespace_parameter(namespace, search_parameter);

    let uri_params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            json!({
                "match":{
                    &column_name: {
                        "query": value
                    }
                }
            })
        })
        .collect::<Vec<serde_json::Value>>();

    json!({
        "bool": {
            "should": uri_params
        }
    })
}
