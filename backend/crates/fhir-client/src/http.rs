use crate::{
    FHIRClient,
    middleware::{Context, Middleware, MiddlewareChain, Next},
    request::{
        self, FHIRCreateResponse, FHIRPatchResponse, FHIRReadResponse, FHIRRequest, FHIRResponse,
        Operation,
    },
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{
        Bundle, CapabilityStatement, OperationOutcome, Parameters, Resource, ResourceType,
    },
    terminology::IssueType,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::VersionId;
use http::HeaderValue;
use reqwest::Url;
use std::{pin::Pin, sync::Arc};

pub struct FHIRHttpState {
    client: reqwest::Client,
    api_url: Url,
    get_access_token: Option<
        Arc<
            dyn Fn() -> Pin<
                    Box<dyn Future<Output = Result<String, OperationOutcomeError>> + Send + Sync>,
                > + Sync
                + Send,
        >,
    >,
}

impl FHIRHttpState {
    pub fn new(
        api_url: &str,
        get_access_token: Option<
            Arc<
                dyn Fn() -> Pin<
                        Box<
                            dyn Future<Output = Result<String, OperationOutcomeError>>
                                + Send
                                + Sync,
                        >,
                    > + Sync
                    + Send,
            >,
        >,
    ) -> Result<Self, OperationOutcomeError> {
        let url =
            Url::parse(api_url).map_err(|_| FHIRHTTPError::UrlParseError(api_url.to_string()))?;
        Ok(FHIRHttpState {
            client: reqwest::Client::new(),
            api_url: url,
            get_access_token,
        })
    }
}

pub struct FHIRHttpClient<CTX> {
    state: Arc<FHIRHttpState>,
    middleware:
        Middleware<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse, OperationOutcomeError>,
}

#[derive(Debug, OperationOutcomeError)]
pub enum FHIRHTTPError {
    #[error(code = "exception", diagnostic = "Reqwest failed.")]
    ReqwestError(#[from] reqwest::Error),
    #[error(code = "not-supported", diagnostic = "Operation not supported.")]
    NotSupported,
    #[fatal(code = "exception", diagnostic = "No response received.")]
    NoResponse,
    #[fatal(
        code = "exception",
        diagnostic = "Invalid url that could not be parsed {arg0}"
    )]
    UrlParseError(String),
    #[error(code = "invalid", diagnostic = "FHIR Deserialization Error.")]
    DeserializeError(#[from] haste_fhir_serialization_json::errors::DeserializeError),
    #[error(code = "invalid", diagnostic = "FHIR Serialization Error.")]
    SerializeError(#[from] haste_fhir_serialization_json::SerializeError),
    #[error(code = "invalid", diagnostic = "FHIR Serialization Error.")]
    JSONSerializeError(#[from] serde_json::Error),
}

fn fhir_parameter_to_query_parameters(http_url: &mut reqwest::Url, parameters: &ParsedParameters) {
    let mut query_parameters = http_url.query_pairs_mut();
    for parameter in parameters.parameters() {
        let parameter = match parameter {
            ParsedParameter::Result(parameter) | ParsedParameter::Resource(parameter) => parameter,
        };

        let mut query_param_name = parameter.name.clone();

        if let Some(chains) = parameter.chains.as_ref() {
            query_param_name = format!("{}.{}", query_param_name, chains.join("."));
        }

        if let Some(modifier) = parameter.modifier.as_ref() {
            query_param_name = format!("{}:{}", query_param_name, modifier);
        }

        query_parameters.append_pair(&query_param_name, parameter.value.join(",").as_str());
    }
}

async fn fhir_request_to_http_request(
    state: &FHIRHttpState,
    request: &FHIRRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let request: Result<reqwest::Request, OperationOutcomeError> = match request {
        FHIRRequest::Read(read_request) => {
            let read_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}",
                    state.api_url.path(),
                    read_request.resource_type.as_ref(),
                    read_request.id
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Read request".to_string()))?;

            let request = state
                .client
                .get(read_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::Create(create_request) => {
            let create_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}",
                    state.api_url.path(),
                    create_request.resource_type.as_ref(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Create request".to_string()))?;

            let body = haste_fhir_serialization_json::to_string(&create_request.resource)
                .map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .post(create_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::Patch(patch_request) => {
            let patch_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}",
                    state.api_url.path(),
                    patch_request.resource_type.as_ref(),
                    patch_request.id
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Patch request".to_string()))?;

            let patch_body =
                serde_json::to_string(&patch_request.patch).map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .patch(patch_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(patch_body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::Transaction(transaction_request) => {
            let body = haste_fhir_serialization_json::to_string(&transaction_request.resource)
                .map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .post(state.api_url.clone())
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::VersionRead(version_request) => {
            let version_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}/_history/{}",
                    state.api_url.path(),
                    version_request.resource_type.as_ref(),
                    version_request.id,
                    version_request.version_id.as_ref(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Patch request".to_string()))?;

            let request = state
                .client
                .get(version_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::UpdateInstance(update_request) => {
            let update_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}",
                    state.api_url.path(),
                    update_request.resource_type.as_ref(),
                    update_request.id
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Update request".to_string()))?;

            let request = state
                .client
                .put(update_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(
                    haste_fhir_serialization_json::to_string(&update_request.resource)
                        .map_err(FHIRHTTPError::from)?,
                )
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::SearchType(search_type_request) => {
            let mut request_url = state
                .api_url
                .join(&format!(
                    "{}/{}",
                    state.api_url.path(),
                    search_type_request.resource_type.as_ref(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("SearchType request".to_string()))?;

            fhir_parameter_to_query_parameters(&mut request_url, &search_type_request.parameters);

            let request = state
                .client
                .get(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::SearchSystem(fhirsearch_system_request) => {
            let mut request_url = state
                .api_url
                .join(state.api_url.path())
                .map_err(|_e| FHIRHTTPError::UrlParseError("SearchSystem request".to_string()))?;

            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirsearch_system_request.parameters,
            );

            let request = state
                .client
                .get(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::ConditionalUpdate(fhirconditional_update_request) => {
            let mut request_url = state
                .api_url
                .join(&format!(
                    "{}/{}",
                    state.api_url.path(),
                    fhirconditional_update_request.resource_type.as_ref(),
                ))
                .map_err(|_e| {
                    FHIRHTTPError::UrlParseError("ConditionalUpdate request".to_string())
                })?;
            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirconditional_update_request.parameters,
            );

            let request = state
                .client
                .put(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(
                    haste_fhir_serialization_json::to_string(
                        &fhirconditional_update_request.resource,
                    )
                    .map_err(FHIRHTTPError::from)?,
                )
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::DeleteInstance(fhirdelete_instance_request) => {
            let delete_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}",
                    state.api_url.path(),
                    fhirdelete_instance_request.resource_type.as_ref(),
                    fhirdelete_instance_request.id
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("DeleteInstance request".to_string()))?;

            let request = state
                .client
                .delete(delete_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::DeleteType(fhirdelete_type_request) => {
            let mut request_url = state
                .api_url
                .join(&format!(
                    "{}/{}",
                    state.api_url.path(),
                    fhirdelete_type_request.resource_type.as_ref(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("DeleteType request".to_string()))?;

            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirdelete_type_request.parameters,
            );

            let request = state
                .client
                .delete(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::DeleteSystem(fhirdelete_system_request) => {
            let mut request_url = state
                .api_url
                .join(state.api_url.path())
                .map_err(|_e| FHIRHTTPError::UrlParseError("DeleteSystem request".to_string()))?;

            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirdelete_system_request.parameters,
            );

            let request = state
                .client
                .delete(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::Capabilities => {
            let request = state
                .client
                .get(format!("{}/metadata", state.api_url))
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::HistoryInstance(fhirhistory_instance_request) => {
            let mut request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}/_history",
                    state.api_url.path(),
                    fhirhistory_instance_request.resource_type.as_ref(),
                    fhirhistory_instance_request.id
                ))
                .map_err(|_e| {
                    FHIRHTTPError::UrlParseError("HistoryInstance request".to_string())
                })?;

            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirhistory_instance_request.parameters,
            );

            let request = state
                .client
                .get(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::HistoryType(fhirhistory_type_request) => {
            let mut request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/_history",
                    state.api_url.path(),
                    fhirhistory_type_request.resource_type.as_ref(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("HistoryType request".to_string()))?;

            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirhistory_type_request.parameters,
            );

            let request = state
                .client
                .get(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::HistorySystem(fhirhistory_system_request) => {
            let mut request_url = state
                .api_url
                .join(&format!("{}/_history", state.api_url.path()))
                .map_err(|_e| FHIRHTTPError::UrlParseError("HistorySystem request".to_string()))?;

            fhir_parameter_to_query_parameters(
                &mut request_url,
                &fhirhistory_system_request.parameters,
            );

            let request = state
                .client
                .get(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::InvokeInstance(fhirinvoke_instance_request) => {
            let request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}/${}",
                    state.api_url.path(),
                    fhirinvoke_instance_request.resource_type.as_ref(),
                    fhirinvoke_instance_request.id,
                    fhirinvoke_instance_request.operation.name(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("InvokeInstance request".to_string()))?;

            // Parameters for invoke are passed in the body as a Parameters resource.
            let body =
                haste_fhir_serialization_json::to_string(&fhirinvoke_instance_request.parameters)
                    .map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .post(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::InvokeType(fhirinvoke_type_request) => {
            let request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/${}",
                    state.api_url.path(),
                    fhirinvoke_type_request.resource_type.as_ref(),
                    fhirinvoke_type_request.operation.name(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("InvokeType request".to_string()))?;

            // Parameters for invoke are passed in the body as a Parameters resource.
            let body =
                haste_fhir_serialization_json::to_string(&fhirinvoke_type_request.parameters)
                    .map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .post(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::InvokeSystem(fhirinvoke_system_request) => {
            let request_url = state
                .api_url
                .join(&format!(
                    "{}/${}",
                    state.api_url.path(),
                    fhirinvoke_system_request.operation.name(),
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("InvokeSystem request".to_string()))?;

            // Parameters for invoke are passed in the body as a Parameters resource.
            let body =
                haste_fhir_serialization_json::to_string(&fhirinvoke_system_request.parameters)
                    .map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .post(request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
        FHIRRequest::Batch(fhirbatch_request) => {
            let body = haste_fhir_serialization_json::to_string(&fhirbatch_request.resource)
                .map_err(FHIRHTTPError::from)?;

            let request = state
                .client
                .post(state.api_url.clone())
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .body(body)
                .build()
                .map_err(FHIRHTTPError::from)?;

            Ok(request)
        }
    };

    let mut request = request?;

    if let Some(get_access_token) = state.get_access_token.as_ref() {
        let token = get_access_token().await?;

        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|_| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Failed to create Authorization header.".to_string(),
                )
            })?,
        );
    }

    Ok(request)
}

async fn check_for_errors(
    status: &reqwest::StatusCode,
    body: Option<&[u8]>,
) -> Result<(), OperationOutcomeError> {
    if !status.is_success() {
        if let Some(body) = body
            && let Ok(operation_outcome) =
                haste_fhir_serialization_json::from_bytes::<OperationOutcome>(&body)
        {
            return Err(OperationOutcomeError::new(None, operation_outcome));
        }

        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("HTTP returned error '{}'.", status),
        ));
    }
    Ok(())
}

async fn http_response_to_fhir_response(
    fhir_request: &FHIRRequest,
    response: reqwest::Response,
) -> Result<FHIRResponse, OperationOutcomeError> {
    match fhir_request {
        FHIRRequest::Read(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;
            Ok(FHIRResponse::Read(FHIRReadResponse {
                resource: Some(resource),
            }))
        }
        FHIRRequest::Create(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;
            Ok(FHIRResponse::Create(FHIRCreateResponse { resource }))
        }
        FHIRRequest::Patch(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;
            Ok(FHIRResponse::Patch(FHIRPatchResponse { resource }))
        }
        FHIRRequest::Transaction(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::Transaction(
                request::FHIRTransactionResponse { resource },
            ))
        }
        FHIRRequest::VersionRead(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;
            Ok(FHIRResponse::VersionRead(
                request::FHIRVersionReadResponse { resource },
            ))
        }
        FHIRRequest::UpdateInstance(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::Update(request::FHIRUpdateResponse {
                resource,
            }))
        }
        FHIRRequest::ConditionalUpdate(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::Update(request::FHIRUpdateResponse {
                resource,
            }))
        }
        FHIRRequest::DeleteInstance(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::DeleteInstance(
                request::FHIRDeleteInstanceResponse { resource },
            ))
        }
        FHIRRequest::DeleteType(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resources = haste_fhir_serialization_json::from_bytes::<Vec<Resource>>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::DeleteType(request::FHIRDeleteTypeResponse {
                resource: resources,
            }))
        }
        FHIRRequest::DeleteSystem(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resources = haste_fhir_serialization_json::from_bytes::<Vec<Resource>>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::DeleteSystem(
                request::FHIRDeleteSystemResponse {
                    resource: resources,
                },
            ))
        }
        FHIRRequest::Capabilities => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let capabilities =
                haste_fhir_serialization_json::from_bytes::<CapabilityStatement>(&body)
                    .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::Capabilities(
                request::FHIRCapabilitiesResponse { capabilities },
            ))
        }
        FHIRRequest::SearchType(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let bundle = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::SearchType(request::FHIRSearchTypeResponse {
                bundle,
            }))
        }
        FHIRRequest::SearchSystem(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let bundle = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::SearchSystem(
                request::FHIRSearchSystemResponse { bundle },
            ))
        }
        FHIRRequest::HistoryInstance(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let bundle = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::HistoryInstance(
                request::FHIRHistoryInstanceResponse { bundle },
            ))
        }
        FHIRRequest::HistoryType(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let bundle = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::HistoryType(
                request::FHIRHistoryTypeResponse { bundle },
            ))
        }
        FHIRRequest::HistorySystem(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let bundle = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::HistorySystem(
                request::FHIRHistorySystemResponse { bundle },
            ))
        }
        FHIRRequest::InvokeInstance(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::InvokeInstance(
                request::FHIRInvokeInstanceResponse { resource },
            ))
        }
        FHIRRequest::InvokeType(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::InvokeType(request::FHIRInvokeTypeResponse {
                resource,
            }))
        }
        FHIRRequest::InvokeSystem(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::InvokeSystem(
                request::FHIRInvokeSystemResponse { resource },
            ))
        }
        FHIRRequest::Batch(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            check_for_errors(&status, Some(&body)).await?;

            let resource = haste_fhir_serialization_json::from_bytes::<Bundle>(&body)
                .map_err(FHIRHTTPError::from)?;

            Ok(FHIRResponse::Batch(request::FHIRBatchResponse { resource }))
        }
    }
}

struct HTTPMiddleware {}
impl HTTPMiddleware {
    fn new() -> Self {
        HTTPMiddleware {}
    }
}
impl<CTX: Send + 'static>
    MiddlewareChain<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse, OperationOutcomeError>
    for HTTPMiddleware
{
    fn call(
        &self,
        state: Arc<FHIRHttpState>,
        context: Context<CTX, FHIRRequest, FHIRResponse>,
        _next: Option<
            Arc<
                Next<
                    Arc<FHIRHttpState>,
                    Context<CTX, FHIRRequest, FHIRResponse>,
                    OperationOutcomeError,
                >,
            >,
        >,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<Context<CTX, FHIRRequest, FHIRResponse>, OperationOutcomeError>,
                > + Send,
        >,
    > {
        Box::pin(async move {
            let http_request = fhir_request_to_http_request(&state, &context.request).await?;
            let response = state
                .client
                .execute(http_request)
                .await
                .map_err(FHIRHTTPError::ReqwestError)?;

            let mut next_context = context;
            let fhir_response =
                http_response_to_fhir_response(&next_context.request, response).await?;
            next_context.response = Some(fhir_response);

            Ok(next_context)
        })
    }
}

impl<CTX: 'static + Send + Sync> FHIRHttpClient<CTX> {
    pub fn new(state: FHIRHttpState) -> Self {
        let middleware = Middleware::new(vec![Box::new(HTTPMiddleware::new())]);
        FHIRHttpClient {
            state: Arc::new(state),
            middleware,
        }
    }
}

impl<CTX: 'static + Send + Sync> FHIRClient<CTX, OperationOutcomeError> for FHIRHttpClient<CTX> {
    async fn request(
        &self,
        ctx: CTX,
        request: crate::request::FHIRRequest,
    ) -> Result<crate::request::FHIRResponse, OperationOutcomeError> {
        let response = self
            .middleware
            .call(self.state.clone(), ctx, request)
            .await?;

        response
            .response
            .ok_or_else(|| FHIRHTTPError::NoResponse.into())
    }

    async fn capabilities(&self, _ctx: CTX) -> Result<CapabilityStatement, OperationOutcomeError> {
        let res = self
            .middleware
            .call(self.state.clone(), _ctx, FHIRRequest::Capabilities)
            .await?;

        match res.response {
            Some(FHIRResponse::Capabilities(capabilities_response)) => {
                Ok(capabilities_response.capabilities)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn search_system(
        &self,
        ctx: CTX,
        parameters: crate::ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::SearchSystem(request::FHIRSearchSystemRequest { parameters }),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::SearchSystem(search_system_response)) => {
                Ok(search_system_response.bundle)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn search_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: crate::ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::SearchType(request::FHIRSearchTypeRequest {
                    resource_type,
                    parameters,
                }),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::SearchType(search_type_response)) => Ok(search_type_response.bundle),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn create(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Create(request::FHIRCreateRequest {
                    resource_type,
                    resource,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Create(create_response)) => Ok(create_response.resource),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn update(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::UpdateInstance(request::FHIRUpdateInstanceRequest {
                    resource_type,
                    id,
                    resource,
                }),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::Update(update_response)) => Ok(update_response.resource),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn conditional_update(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: crate::ParsedParameters,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::ConditionalUpdate(request::FHIRConditionalUpdateRequest {
                    resource_type,
                    parameters,
                    resource,
                }),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::Update(update_response)) => Ok(update_response.resource),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn patch(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        patch: json_patch::Patch,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Patch(request::FHIRPatchRequest {
                    resource_type,
                    id,
                    patch,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Patch(patch_response)) => Ok(patch_response.resource),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn read(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Read(request::FHIRReadRequest { resource_type, id }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Read(read_response)) => Ok(read_response.resource),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn vread(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        version_id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::VersionRead(request::FHIRVersionReadRequest {
                    resource_type,
                    id,
                    version_id: VersionId::new(version_id),
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::VersionRead(version_read_response)) => {
                Ok(Some(version_read_response.resource))
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn delete_instance(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
    ) -> Result<(), OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::DeleteInstance(request::FHIRDeleteInstanceRequest {
                    resource_type,
                    id,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::DeleteInstance(_delete_instance_response)) => Ok(()),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn delete_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: crate::ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::DeleteType(request::FHIRDeleteTypeRequest {
                    resource_type,
                    parameters,
                }),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::DeleteType(_delete_type_response)) => Ok(()),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn delete_system(
        &self,
        ctx: CTX,
        parameters: crate::ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::DeleteSystem(request::FHIRDeleteSystemRequest { parameters }),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::DeleteSystem(_delete_system_response)) => Ok(()),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn history_system(
        &self,
        ctx: CTX,
        parameters: crate::ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::HistorySystem(request::FHIRHistorySystemRequest { parameters }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::HistorySystem(history_system_response)) => {
                Ok(history_system_response.bundle)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn history_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: crate::ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::HistoryType(request::FHIRHistoryTypeRequest {
                    resource_type,
                    parameters,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::HistoryType(history_type_response)) => {
                Ok(history_type_response.bundle)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn history_instance(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        parameters: crate::ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::HistoryInstance(request::FHIRHistoryInstanceRequest {
                    resource_type,
                    id,
                    parameters,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::HistoryInstance(history_instance_response)) => {
                Ok(history_instance_response.bundle)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn invoke_instance(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        operation: String,
        parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::InvokeInstance(request::FHIRInvokeInstanceRequest {
                    resource_type,
                    id,
                    operation: Operation::new(&operation).map_err(|_e| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "invalid operation".to_string(),
                        )
                    })?,
                    parameters,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::InvokeInstance(invoke_instance_response)) => {
                Ok(invoke_instance_response.resource)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn invoke_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        operation: String,
        parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::InvokeType(request::FHIRInvokeTypeRequest {
                    resource_type,
                    operation: Operation::new(&operation).map_err(|_e| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "invalid operation".to_string(),
                        )
                    })?,
                    parameters,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::InvokeType(invoke_type_response)) => {
                Ok(invoke_type_response.resource)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn invoke_system(
        &self,
        ctx: CTX,
        operation: String,
        parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::InvokeSystem(request::FHIRInvokeSystemRequest {
                    operation: Operation::new(&operation).map_err(|_e| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "invalid operation".to_string(),
                        )
                    })?,
                    parameters,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::InvokeSystem(invoke_system_response)) => {
                Ok(invoke_system_response.resource)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn transaction(&self, ctx: CTX, bundle: Bundle) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Transaction(request::FHIRTransactionRequest { resource: bundle }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Transaction(transaction_response)) => {
                Ok(transaction_response.resource)
            }
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn batch(&self, ctx: CTX, bundle: Bundle) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Batch(request::FHIRBatchRequest { resource: bundle }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Batch(batch_response)) => Ok(batch_response.resource),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }
}
