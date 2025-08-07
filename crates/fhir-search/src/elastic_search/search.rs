use oxidized_fhir_client::url::{Parameter, ParsedParameter};
use oxidized_fhir_model::r4::types::{ResourceType, SearchParameter};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use oxidized_fhir_repository::VersionIdRef;
use serde_json::json;

use crate::SearchRequest;

#[derive(OperationOutcomeError, Debug)]
pub enum QueryBuildError {
    #[error(
        code = "not-found",
        diagnostic = "Search parameter with name '{arg1}' not found for resource type '{arg0:?}'"
    )]
    MissingParameter(ResourceType, String),
    #[error(
        code = "invalid",
        diagnostic = "Unsupported search request type or parameter: {arg0}"
    )]
    UnsupportedParameter(String),
    #[error(code = "invalid", diagnostic = "Invalid parameter value: '{arg0}'")]
    InvalidParameterValue(String),
}

fn build_query(
    search_param: SearchParameter,
    parsed_parameter: Parameter,
) -> Result<Vec<serde_json::Value>, QueryBuildError> {
    match search_param.type_.value.as_ref().map(|s| s.as_str()) {
        Some("number") => parsed_parameter
            .value
            .iter()
            .map(|value| {
                let v = value
                    .parse::<f64>()
                    .map_err(|e| QueryBuildError::InvalidParameterValue(value.to_string()))?;
                let k = json!({
                    "range": {
                        "field": search_param.url.value.as_ref().unwrap(),
                        "gte": v,
                        "lte": v
                    }
                });

                Ok(k)
            })
            .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>(),
        Some("string") => {
            todo!()
        }
        _ => todo!(),
    }
}

fn build_elastic_search_query(
    request: &SearchRequest,
) -> Result<Vec<VersionIdRef<'static>>, QueryBuildError> {
    match request {
        SearchRequest::TypeSearch(type_search_request) => {
            let resource_type = &type_search_request.resource_type;
            let parameters = &type_search_request.parameters;

            for parameter in parameters.iter() {
                match parameter {
                    ParsedParameter::Resource(resource_param) => {
                        let search_param =
                            oxidized_artifacts::search_parameters::get_search_parameter_for_name(
                                &resource_type,
                                &resource_param.name,
                            )
                            .ok_or_else(|| {
                                QueryBuildError::MissingParameter(
                                    resource_type.clone(),
                                    resource_param.name.to_string(),
                                )
                            })?;
                    }
                    ParsedParameter::Result(result_param) => {
                        return Err(QueryBuildError::UnsupportedParameter(
                            result_param.name.to_string(),
                        ));
                    }
                }
            }

            todo!();
        }
        _ => {
            return Err(QueryBuildError::UnsupportedParameter(
                "Unsupported search request type".to_string(),
            ));
        }
    }
}
