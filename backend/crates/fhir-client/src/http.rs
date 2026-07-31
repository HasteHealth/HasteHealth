use crate::{
    FHIRClient,
    middleware::{Context, Middleware, MiddlewareChain, Next},
    request::{
        self, CompartmentRequest, DeleteRequest, DeleteResponse, FHIRBatchRequest,
        FHIRBatchResponse, FHIRConditionalUpdateRequest, FHIRCreateRequest, FHIRCreateResponse,
        FHIRDeleteInstanceRequest, FHIRDeleteSystemRequest, FHIRDeleteTypeRequest,
        FHIRHistoryInstanceRequest, FHIRHistorySystemRequest, FHIRHistoryTypeRequest,
        FHIRInvokeInstanceRequest, FHIRInvokeSystemRequest, FHIRInvokeTypeRequest,
        FHIRPatchRequest, FHIRPatchResponse, FHIRReadRequest, FHIRReadResponse, FHIRRequest,
        FHIRResponse, FHIRSearchSystemRequest, FHIRSearchTypeRequest, FHIRTransactionRequest,
        FHIRUpdateInstanceRequest, FHIRVersionReadRequest, HistoryRequest, HistoryResponse,
        InvocationRequest, InvokeResponse, Operation, SearchRequest, SearchResponse, UpdateRequest,
    },
    url::{ParsedParameter, ParsedParameters},
};
use derivative::Derivative;
use haste_fhir_model::r4::generated::{
    resources::{
        Bundle, CapabilityStatement, OperationOutcome, Parameters, Resource, ResourceType,
    },
    terminology::IssueType,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::VersionId;
use http::HeaderValue;
use reqwest::{Request, RequestBuilder, Url};
use std::future::Future;
use std::{fmt::Debug, pin::Pin, sync::Arc};

type AccessToken = dyn Fn() -> Pin<Box<dyn Future<Output = Result<String, OperationOutcomeError>> + Send + Sync>>
    + Sync
    + Send;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FHIRHttpState {
    #[derivative(Debug = "ignore")]
    client: reqwest::Client,
    api_url: Url,
    #[derivative(Debug = "ignore")]
    get_access_token: Option<Arc<AccessToken>>,
}

impl FHIRHttpState {
    /// Creates a new HTTP client.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the provided `api_url` is invalid
    /// or if the client cannot be initialized.
    pub fn new(
        api_url: &str,
        get_access_token: Option<Arc<AccessToken>>,
    ) -> Result<Self, OperationOutcomeError> {
        let mut url =
            Url::parse(api_url).map_err(|_| FHIRHTTPError::UrlParseError(api_url.to_string()))?;

        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }

        Ok(FHIRHttpState {
            client: reqwest::Client::new(),
            api_url: url,
            get_access_token,
        })
    }
}

pub struct FHIRHttpClient<CTX: Debug> {
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
    #[error(code = "invalid", diagnostic = "FHIR Deserialization Error '{arg0}'.")]
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
            query_param_name = format!("{query_param_name}.{}", chains.join("."));
        }

        if let Some(modifier) = parameter.modifier.as_ref() {
            query_param_name = format!("{query_param_name}:{modifier}");
        }

        query_parameters.append_pair(&query_param_name, parameter.value.join(",").as_str());
    }
}

fn build_request(builder: RequestBuilder) -> Result<Request, OperationOutcomeError> {
    builder
        .header("Accept", "application/fhir+json")
        .header("Content-Type", "application/fhir+json, application/json")
        .build()
        .map_err(FHIRHTTPError::from)
        .map_err(Into::into)
}

fn build_get(state: &FHIRHttpState, url: reqwest::Url) -> Result<Request, OperationOutcomeError> {
    build_request(state.client.get(url))
}

fn build_post(
    state: &FHIRHttpState,
    url: reqwest::Url,
    body: String,
) -> Result<Request, OperationOutcomeError> {
    build_request(state.client.post(url).body(body))
}

fn build_put(
    state: &FHIRHttpState,
    url: reqwest::Url,
    body: String,
) -> Result<Request, OperationOutcomeError> {
    build_request(state.client.put(url).body(body))
}

fn build_patch(
    state: &FHIRHttpState,
    url: reqwest::Url,
    body: String,
) -> Result<Request, OperationOutcomeError> {
    build_request(state.client.patch(url).body(body))
}

fn build_delete(
    state: &FHIRHttpState,
    url: reqwest::Url,
) -> Result<Request, OperationOutcomeError> {
    build_request(state.client.delete(url))
}

fn serialize_json<T: serde::Serialize>(value: &T) -> Result<String, OperationOutcomeError> {
    serde_json::to_string(value)
        .map_err(FHIRHTTPError::from)
        .map_err(Into::into)
}

fn fhir_request_to_http_request<'a>(
    state: &'a FHIRHttpState,
    request: &'a FHIRRequest,
) -> Pin<Box<dyn Future<Output = Result<Request, OperationOutcomeError>> + Send + 'a>> {
    Box::pin(async move {
        let request = match request {
            FHIRRequest::Read(request) => request_from_read(state, request),
            FHIRRequest::Compartment(request) => request_from_compartment(state, request).await,
            FHIRRequest::Create(request) => request_from_create(state, request),
            FHIRRequest::Patch(request) => request_from_patch(state, request),
            FHIRRequest::Transaction(request) => request_from_transaction(state, request),
            FHIRRequest::VersionRead(request) => request_from_version_read(state, request),
            FHIRRequest::Update(request) => request_from_update(state, request),
            FHIRRequest::Search(request) => request_from_search(state, request),
            FHIRRequest::Delete(request) => request_from_delete(state, request),
            FHIRRequest::Capabilities => request_from_capabilities(state),
            FHIRRequest::History(request) => request_from_history(state, request),
            FHIRRequest::Invocation(request) => request_from_invocation(state, request),
            FHIRRequest::Batch(request) => request_from_batch(state, request),
        };

        let mut request = request?;

        if let Some(get_access_token) = state.get_access_token.as_ref() {
            let token = get_access_token().await?;

            request.headers_mut().insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|_| {
                    OperationOutcomeError::error(
                        IssueType::invalid(),
                        "Failed to create Authorization header.".to_string(),
                    )
                })?,
            );
        }

        Ok(request)
    })
}

fn request_from_read(
    state: &FHIRHttpState,
    read_request: &FHIRReadRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let read_request_url = state
        .api_url
        .join(&format!(
            "{}/{}",
            read_request.resource_type.as_ref(),
            read_request.id
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("Read request".to_string()))?;

    build_get(state, read_request_url)
}

fn request_from_compartment<'a>(
    state: &'a FHIRHttpState,
    compartment_request: &'a CompartmentRequest,
) -> Pin<Box<dyn Future<Output = Result<reqwest::Request, OperationOutcomeError>> + Send + 'a>> {
    Box::pin(async move {
        let compartment_url = state
            .api_url
            .join(&format!(
                "{}/{}",
                compartment_request.resource_type.as_ref(),
                compartment_request.id
            ))
            .map_err(|_| FHIRHTTPError::UrlParseError("Compartment request".to_string()))?;

        let compartment_state = FHIRHttpState {
            api_url: compartment_url,
            client: state.client.clone(),
            get_access_token: state.get_access_token.clone(),
        };

        fhir_request_to_http_request(&compartment_state, &compartment_request.request).await
    })
}

fn request_from_create(
    state: &FHIRHttpState,
    create_request: &FHIRCreateRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let create_request_url = state
        .api_url
        .join(&format!("{}", create_request.resource_type.as_ref(),))
        .map_err(|_| FHIRHTTPError::UrlParseError("Create request".to_string()))?;

    let body = serialize_json(&create_request.resource)?;

    build_post(state, create_request_url, body)
}

fn request_from_patch(
    state: &FHIRHttpState,
    patch_request: &FHIRPatchRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let patch_request_url = state
        .api_url
        .join(&format!(
            "{}/{}",
            patch_request.resource_type.as_ref(),
            patch_request.id
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("Patch request".to_string()))?;

    let body = serialize_json(&patch_request.patch)?;

    build_patch(state, patch_request_url, body)
}

fn request_from_transaction(
    state: &FHIRHttpState,
    transaction_request: &FHIRTransactionRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let body = serialize_json(&transaction_request.resource)?;

    build_post(state, state.api_url.clone(), body)
}

fn request_from_version_read(
    state: &FHIRHttpState,
    version_request: &FHIRVersionReadRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let version_request_url = state
        .api_url
        .join(&format!(
            "{}/{}/_history/{}",
            version_request.resource_type.as_ref(),
            version_request.id,
            version_request.version_id.as_ref(),
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("Version read request".to_string()))?;

    build_get(state, version_request_url)
}

fn request_from_capabilities(
    state: &FHIRHttpState,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let url = state
        .api_url
        .join("metadata")
        .map_err(|_| FHIRHTTPError::UrlParseError("Capabilities request".to_string()))?;

    build_get(state, url)
}

fn request_from_batch(
    state: &FHIRHttpState,
    batch_request: &FHIRBatchRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let body = serialize_json(&batch_request.resource)?;

    build_post(state, state.api_url.clone(), body)
}

fn request_from_update(
    state: &FHIRHttpState,
    update_request: &UpdateRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    match update_request {
        UpdateRequest::Instance(request) => request_from_update_instance(state, request),
        UpdateRequest::Conditional(request) => request_from_update_conditional(state, request),
    }
}

fn request_from_update_instance(
    state: &FHIRHttpState,
    update_request: &FHIRUpdateInstanceRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let update_request_url = state
        .api_url
        .join(&format!(
            "{}/{}",
            update_request.resource_type.as_ref(),
            update_request.id
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("Update request".to_string()))?;

    let body = serialize_json(&update_request.resource)?;

    build_put(state, update_request_url, body)
}

fn request_from_update_conditional(
    state: &FHIRHttpState,
    update_request: &FHIRConditionalUpdateRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state
        .api_url
        .join(&format!("{}", update_request.resource_type.as_ref(),))
        .map_err(|_| FHIRHTTPError::UrlParseError("ConditionalUpdate request".to_string()))?;

    fhir_parameter_to_query_parameters(&mut request_url, &update_request.parameters);

    let body = serialize_json(&update_request.resource)?;

    build_put(state, request_url, body)
}

fn request_from_search(
    state: &FHIRHttpState,
    search_request: &SearchRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    match search_request {
        SearchRequest::Type(request) => request_from_search_type(state, request),
        SearchRequest::System(request) => request_from_search_system(state, request),
    }
}

fn request_from_search_type(
    state: &FHIRHttpState,
    search_request: &FHIRSearchTypeRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state
        .api_url
        .join(&format!("{}", search_request.resource_type.as_ref(),))
        .map_err(|_| FHIRHTTPError::UrlParseError("SearchType request".to_string()))?;

    fhir_parameter_to_query_parameters(&mut request_url, &search_request.parameters);

    build_get(state, request_url)
}

fn request_from_search_system(
    state: &FHIRHttpState,
    search_request: &FHIRSearchSystemRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state.api_url.clone();

    fhir_parameter_to_query_parameters(&mut request_url, &search_request.parameters);

    build_get(state, request_url)
}

fn request_from_delete(
    state: &FHIRHttpState,
    delete_request: &DeleteRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    match delete_request {
        DeleteRequest::Instance(request) => request_from_delete_instance(state, request),
        DeleteRequest::Type(request) => request_from_delete_type(state, request),
        DeleteRequest::System(request) => request_from_delete_system(state, request),
    }
}

fn request_from_delete_instance(
    state: &FHIRHttpState,
    delete_request: &FHIRDeleteInstanceRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let delete_request_url = state
        .api_url
        .join(&format!(
            "{}/{}",
            delete_request.resource_type.as_ref(),
            delete_request.id
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("DeleteInstance request".to_string()))?;

    build_delete(state, delete_request_url)
}

fn request_from_delete_type(
    state: &FHIRHttpState,
    delete_request: &FHIRDeleteTypeRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state
        .api_url
        .join(&format!("{}", delete_request.resource_type.as_ref(),))
        .map_err(|_| FHIRHTTPError::UrlParseError("DeleteType request".to_string()))?;

    fhir_parameter_to_query_parameters(&mut request_url, &delete_request.parameters);

    build_delete(state, request_url)
}

fn request_from_delete_system(
    state: &FHIRHttpState,
    delete_request: &FHIRDeleteSystemRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state.api_url.clone();

    fhir_parameter_to_query_parameters(&mut request_url, &delete_request.parameters);

    build_delete(state, request_url)
}

fn request_from_history(
    state: &FHIRHttpState,
    history_request: &HistoryRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    match history_request {
        HistoryRequest::Instance(request) => request_from_history_instance(state, request),
        HistoryRequest::Type(request) => request_from_history_type(state, request),
        HistoryRequest::System(request) => request_from_history_system(state, request),
    }
}

fn request_from_history_instance(
    state: &FHIRHttpState,
    history_request: &FHIRHistoryInstanceRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state
        .api_url
        .join(&format!(
            "{}/{}/_history",
            history_request.resource_type.as_ref(),
            history_request.id
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("HistoryInstance request".to_string()))?;

    fhir_parameter_to_query_parameters(&mut request_url, &history_request.parameters);

    build_get(state, request_url)
}

fn request_from_history_type(
    state: &FHIRHttpState,
    history_request: &FHIRHistoryTypeRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state
        .api_url
        .join(&format!(
            "{}/_history",
            history_request.resource_type.as_ref(),
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("HistoryType request".to_string()))?;

    fhir_parameter_to_query_parameters(&mut request_url, &history_request.parameters);

    build_get(state, request_url)
}

fn request_from_history_system(
    state: &FHIRHttpState,
    history_request: &FHIRHistorySystemRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request_url = state
        .api_url
        .join(&format!("_history"))
        .map_err(|_| FHIRHTTPError::UrlParseError("HistorySystem request".to_string()))?;

    fhir_parameter_to_query_parameters(&mut request_url, &history_request.parameters);

    build_get(state, request_url)
}

fn request_from_invocation(
    state: &FHIRHttpState,
    invocation_request: &InvocationRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    match invocation_request {
        InvocationRequest::Instance(request) => request_from_invocation_instance(state, request),
        InvocationRequest::Type(request) => request_from_invocation_type(state, request),
        InvocationRequest::System(request) => request_from_invocation_system(state, request),
    }
}

fn request_from_invocation_instance(
    state: &FHIRHttpState,
    invocation_request: &FHIRInvokeInstanceRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let request_url = state
        .api_url
        .join(&format!(
            "{}/{}/${}",
            invocation_request.resource_type.as_ref(),
            invocation_request.id,
            invocation_request.operation.name(),
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("InvokeInstance request".to_string()))?;

    let body = serialize_json(&invocation_request.parameters)?;

    build_post(state, request_url, body)
}

fn request_from_invocation_type(
    state: &FHIRHttpState,
    invocation_request: &FHIRInvokeTypeRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let request_url = state
        .api_url
        .join(&format!(
            "{}/${}",
            invocation_request.resource_type.as_ref(),
            invocation_request.operation.name(),
        ))
        .map_err(|_| FHIRHTTPError::UrlParseError("InvokeType request".to_string()))?;

    let body = serialize_json(&invocation_request.parameters)?;

    build_post(state, request_url, body)
}

fn request_from_invocation_system(
    state: &FHIRHttpState,
    invocation_request: &FHIRInvokeSystemRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let request_url = state
        .api_url
        .join(&format!("${}", invocation_request.operation.name(),))
        .map_err(|_| FHIRHTTPError::UrlParseError("InvokeSystem request".to_string()))?;

    let body = serialize_json(&invocation_request.parameters)?;

    build_post(state, request_url, body)
}

enum FHIRResponseRequest<'a> {
    Read,
    Create,
    Patch,
    Transaction,
    VersionRead,
    Update(&'a UpdateRequest),
    Delete(&'a DeleteRequest),
    Capabilities,
    Search(&'a SearchRequest),
    History(&'a HistoryRequest),
    Invocation(&'a InvocationRequest),
    Batch,
}

impl<'a> FHIRResponseRequest<'a> {
    const fn response_request(request: &'a FHIRRequest) -> Self {
        match request {
            FHIRRequest::Compartment(request) => Self::response_request(&request.request),
            FHIRRequest::Read(_) => Self::Read,
            FHIRRequest::Create(_) => Self::Create,
            FHIRRequest::Patch(_) => Self::Patch,
            FHIRRequest::Transaction(_) => Self::Transaction,
            FHIRRequest::VersionRead(_) => Self::VersionRead,
            FHIRRequest::Update(request) => Self::Update(request),
            FHIRRequest::Delete(request) => Self::Delete(request),
            FHIRRequest::Capabilities => Self::Capabilities,
            FHIRRequest::Search(request) => Self::Search(request),
            FHIRRequest::History(request) => Self::History(request),
            FHIRRequest::Invocation(request) => Self::Invocation(request),
            FHIRRequest::Batch(_) => Self::Batch,
        }
    }
}

fn http_response_to_fhir_response<'a>(
    fhir_request: &'a FHIRRequest,
    response: reqwest::Response,
) -> Pin<Box<dyn Future<Output = Result<FHIRResponse, OperationOutcomeError>> + Send + 'a>> {
    Box::pin(async move {
        let request = FHIRResponseRequest::response_request(fhir_request);
        let body = read_response(response).await?;

        build_response(request, &body)
    })
}

fn check_for_errors(
    status: reqwest::StatusCode,
    body: Option<&[u8]>,
) -> Result<(), OperationOutcomeError> {
    if !status.is_success() {
        if let Some(body) = body
            && let Ok(operation_outcome) = serde_json::from_slice::<OperationOutcome>(body)
        {
            return Err(OperationOutcomeError::new(None, operation_outcome));
        }

        return Err(OperationOutcomeError::error(
            IssueType::exception(),
            format!("HTTP returned error '{status}'."),
        ));
    }
    Ok(())
}

async fn read_response(response: reqwest::Response) -> Result<bytes::Bytes, OperationOutcomeError> {
    let status = response.status();

    let body = response
        .bytes()
        .await
        .map_err(FHIRHTTPError::ReqwestError)?;

    check_for_errors(status, Some(&body))?;

    Ok(body)
}

fn build_response(
    request: FHIRResponseRequest<'_>,
    body: &[u8],
) -> Result<FHIRResponse, OperationOutcomeError> {
    match request {
        FHIRResponseRequest::Read => build_read_response(body),
        FHIRResponseRequest::Create => build_create_response(body),
        FHIRResponseRequest::Patch => build_patch_response(body),
        FHIRResponseRequest::Transaction => build_transaction_response(body),
        FHIRResponseRequest::VersionRead => build_version_read_response(body),
        FHIRResponseRequest::Update(request) => build_update_response(request, body),
        FHIRResponseRequest::Delete(request) => build_delete_response(request, body),
        FHIRResponseRequest::Capabilities => build_capabilities_response(body),
        FHIRResponseRequest::Search(request) => build_search_response(request, body),
        FHIRResponseRequest::History(request) => build_history_response(request, body),
        FHIRResponseRequest::Invocation(request) => build_invocation_response(request, body),
        FHIRResponseRequest::Batch => build_batch_response(body),
    }
}

fn deserialize<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, OperationOutcomeError> {
    serde_json::from_slice(body)
        .map_err(FHIRHTTPError::from)
        .map_err(Into::into)
}

fn build_read_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Read(FHIRReadResponse {
        resource: Some(deserialize(body)?),
    }))
}

fn build_create_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Create(FHIRCreateResponse {
        resource: deserialize(body)?,
    }))
}

fn build_patch_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Patch(FHIRPatchResponse {
        resource: deserialize(body)?,
    }))
}

fn build_transaction_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Transaction(
        request::FHIRTransactionResponse {
            resource: deserialize(body)?,
        },
    ))
}

fn build_batch_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Batch(FHIRBatchResponse {
        resource: deserialize(body)?,
    }))
}

fn build_version_read_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::VersionRead(
        request::FHIRVersionReadResponse {
            resource: deserialize(body)?,
        },
    ))
}

fn build_update_response(
    _request: &UpdateRequest,
    body: &[u8],
) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Update(request::FHIRUpdateResponse {
        resource: deserialize(body)?,
    }))
}

fn build_capabilities_response(body: &[u8]) -> Result<FHIRResponse, OperationOutcomeError> {
    Ok(FHIRResponse::Capabilities(
        request::FHIRCapabilitiesResponse {
            capabilities: deserialize(body)?,
        },
    ))
}

fn build_search_response(
    request: &SearchRequest,
    body: &[u8],
) -> Result<FHIRResponse, OperationOutcomeError> {
    let bundle = deserialize(body)?;

    match request {
        SearchRequest::Type(_) => Ok(FHIRResponse::Search(SearchResponse::Type(
            request::FHIRSearchTypeResponse { bundle },
        ))),
        SearchRequest::System(_) => Ok(FHIRResponse::Search(SearchResponse::System(
            request::FHIRSearchSystemResponse { bundle },
        ))),
    }
}

fn build_delete_response(
    request: &DeleteRequest,
    body: &[u8],
) -> Result<FHIRResponse, OperationOutcomeError> {
    match request {
        DeleteRequest::Instance(_) => Ok(FHIRResponse::Delete(DeleteResponse::Instance(Box::new(
            request::FHIRDeleteInstanceResponse {
                resource: deserialize(body)?,
            },
        )))),

        DeleteRequest::Type(_) => Ok(FHIRResponse::Delete(DeleteResponse::Type(
            request::FHIRDeleteTypeResponse {},
        ))),

        DeleteRequest::System(_) => Ok(FHIRResponse::Delete(DeleteResponse::System(
            request::FHIRDeleteSystemResponse {},
        ))),
    }
}

fn build_history_response(
    request: &HistoryRequest,
    body: &[u8],
) -> Result<FHIRResponse, OperationOutcomeError> {
    let bundle = deserialize(body)?;

    match request {
        HistoryRequest::Instance(_) => Ok(FHIRResponse::History(HistoryResponse::Instance(
            request::FHIRHistoryInstanceResponse { bundle },
        ))),
        HistoryRequest::Type(_) => Ok(FHIRResponse::History(HistoryResponse::Type(
            request::FHIRHistoryTypeResponse { bundle },
        ))),
        HistoryRequest::System(_) => Ok(FHIRResponse::History(HistoryResponse::System(
            request::FHIRHistorySystemResponse { bundle },
        ))),
    }
}

fn build_invocation_response(
    request: &InvocationRequest,
    body: &[u8],
) -> Result<FHIRResponse, OperationOutcomeError> {
    let resource = deserialize(body)?;

    match request {
        InvocationRequest::Instance(_) => Ok(FHIRResponse::Invoke(InvokeResponse::Instance(
            request::FHIRInvokeInstanceResponse { resource },
        ))),
        InvocationRequest::Type(_) => Ok(FHIRResponse::Invoke(InvokeResponse::Type(
            request::FHIRInvokeTypeResponse { resource },
        ))),
        InvocationRequest::System(_) => Ok(FHIRResponse::Invoke(InvokeResponse::System(
            request::FHIRInvokeSystemResponse { resource },
        ))),
    }
}

struct HTTPMiddleware {}
impl HTTPMiddleware {
    fn new() -> Self {
        HTTPMiddleware {}
    }
}
impl<CTX: Send + 'static + Debug>
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

impl<CTX: 'static + Send + Sync + Debug> FHIRHttpClient<CTX> {
    #[must_use]
    pub fn new(state: FHIRHttpState) -> Self {
        let middleware = Middleware::new(vec![Box::new(HTTPMiddleware::new())]);
        FHIRHttpClient {
            state: Arc::new(state),
            middleware,
        }
    }
}

impl<CTX: 'static + Send + Sync + Debug> FHIRClient<CTX, OperationOutcomeError>
    for FHIRHttpClient<CTX>
{
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

    async fn capabilities(&self, ctx: CTX) -> Result<CapabilityStatement, OperationOutcomeError> {
        let res = self
            .middleware
            .call(self.state.clone(), ctx, FHIRRequest::Capabilities)
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
                FHIRRequest::Search(SearchRequest::System(request::FHIRSearchSystemRequest {
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::Search(SearchResponse::System(search_system_response))) => {
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
                FHIRRequest::Search(SearchRequest::Type(request::FHIRSearchTypeRequest {
                    resource_type,
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::Search(SearchResponse::Type(search_type_response))) => {
                Ok(search_type_response.bundle)
            }
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
                FHIRRequest::Update(UpdateRequest::Instance(
                    request::FHIRUpdateInstanceRequest {
                        resource_type,
                        id,
                        resource,
                    },
                )),
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
                FHIRRequest::Update(UpdateRequest::Conditional(
                    request::FHIRConditionalUpdateRequest {
                        resource_type,
                        parameters,
                        resource,
                    },
                )),
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
                FHIRRequest::Delete(DeleteRequest::Instance(
                    request::FHIRDeleteInstanceRequest { resource_type, id },
                )),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Delete(_delete_instance_response)) => Ok(()),
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
                FHIRRequest::Delete(DeleteRequest::Type(request::FHIRDeleteTypeRequest {
                    resource_type,
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::Delete(_delete_type_response)) => Ok(()),
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
                FHIRRequest::Delete(DeleteRequest::System(request::FHIRDeleteSystemRequest {
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::Delete(_delete_system_response)) => Ok(()),
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
                FHIRRequest::History(HistoryRequest::System(request::FHIRHistorySystemRequest {
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::History(HistoryResponse::System(history_system_response))) => {
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
                FHIRRequest::History(HistoryRequest::Type(request::FHIRHistoryTypeRequest {
                    resource_type,
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::History(HistoryResponse::Type(history_type_response))) => {
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
                FHIRRequest::History(HistoryRequest::Instance(
                    request::FHIRHistoryInstanceRequest {
                        resource_type,
                        id,
                        parameters,
                    },
                )),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::History(HistoryResponse::Instance(history_instance_response))) => {
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
                FHIRRequest::Invocation(InvocationRequest::Instance(
                    request::FHIRInvokeInstanceRequest {
                        resource_type,
                        id,
                        operation: Operation::new(&operation),
                        parameters,
                    },
                )),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Invoke(InvokeResponse::Instance(invoke_instance_response))) => {
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
                FHIRRequest::Invocation(InvocationRequest::Type(request::FHIRInvokeTypeRequest {
                    resource_type,
                    operation: Operation::new(&operation),
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Invoke(InvokeResponse::Type(invoke_type_response))) => {
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
                FHIRRequest::Invocation(InvocationRequest::System(
                    request::FHIRInvokeSystemRequest {
                        operation: Operation::new(&operation),
                        parameters,
                    },
                )),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Invoke(InvokeResponse::System(invoke_system_response))) => {
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
