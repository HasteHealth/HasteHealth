use oxidized_fhir_client::url::ParsedParameter;
use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use oxidized_fhir_repository::VersionIdRef;

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
