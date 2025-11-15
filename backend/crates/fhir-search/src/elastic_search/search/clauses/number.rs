use crate::{elastic_search::search::QueryBuildError, indexing_conversion::get_decimal_range};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn number(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let v = value
                .parse::<f64>()
                .map_err(|_e| QueryBuildError::InvalidParameterValue(value.to_string()))?;

            let range = get_decimal_range(v);

            let k = json!({
                "range": {
                    search_param.url.value.as_ref().unwrap(): {
                        "gte": range.start,
                        "lte": range.end
                    }
                }
            });

            Ok(k)
        })
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}
