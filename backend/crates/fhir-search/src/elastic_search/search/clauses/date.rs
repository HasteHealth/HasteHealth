use crate::{elastic_search::search::QueryBuildError, indexing_conversion::date_time_range};
use oxidized_fhir_client::url::Parameter;
use oxidized_fhir_model::r4::{
    datetime::parse_datetime,
    generated::resources::SearchParameter,
};
use serde_json::json;

pub fn date(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let date_time = parse_datetime(value).map_err(|_e| 
                QueryBuildError::InvalidDateFormat(value.to_string()))?;
            let date_range = date_time_range(&date_time).map_err(|_e| 
                QueryBuildError::InvalidDateFormat(value.to_string()))?;

            Ok(json!({
                "nested": {
                    "path": search_param.url.value.as_ref().unwrap(),
                    "query": {
                        "bool": {
                            "must": [
                                {
                                    "range": {
                                        search_param.url.value.as_ref().unwrap().to_string() + ".start": {
                                            "gte": date_range.start
                                        }
                                    }
                                },
                                {
                                    "range": {
                                        search_param.url.value.as_ref().unwrap().to_string() + ".end": {
                                            "lte": date_range.end
                                        }
                                    }
                                }
                            ]
                        }
                    }
                }
            }))
        })
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}
