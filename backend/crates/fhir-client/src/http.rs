use crate::{
    FHIRClient,
    middleware::{Context, Middleware, MiddlewareChain, Next},
    request::{self, FHIRReadResponse, FHIRRequest, FHIRResponse},
};
use http::HeaderValue;
use oxidized_fhir_model::r4::generated::{
    resources::{CapabilityStatement, OperationOutcome, Parameters, Resource, ResourceType},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use reqwest::Url;
use std::{pin::Pin, sync::Arc};

pub struct FHIRHttpState {
    client: reqwest::Client,
    api_url: Url,
    get_access_token: Option<
        Arc<
            dyn Fn() -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>
                + Sync
                + Send,
        >,
    >,
}

impl FHIRHttpState {
    pub fn new(
        api_url: &str,
        get_access_token: Option<
            Arc<
                dyn Fn() -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + Sync>>
                    + Sync
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
    DeserializeError(#[from] oxidized_fhir_serialization_json::errors::DeserializeError),
}

async fn fhir_request_to_http_request(
    state: &FHIRHttpState,
    request: &FHIRRequest,
) -> Result<reqwest::Request, OperationOutcomeError> {
    let mut request = match request {
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
        _ => Err(FHIRHTTPError::NotSupported),
    }?;

    if let Some(get_access_token) = state.get_access_token.as_ref() {
        let token = get_access_token()
            .await
            .map_err(|e| OperationOutcomeError::error(IssueType::Forbidden(None), e))?;
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
                oxidized_fhir_serialization_json::from_bytes::<OperationOutcome>(&body)
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

            let resource = oxidized_fhir_serialization_json::from_bytes::<Resource>(&body)
                .map_err(FHIRHTTPError::from)?;
            Ok(FHIRResponse::Read(FHIRReadResponse { resource }))
        }
        FHIRRequest::Create(_) => {
            todo!();
            // FHIRResponse::Create(request::FHIRCreateResponse { resource:  })
        }
        FHIRRequest::VersionRead(_) => todo!(),
        FHIRRequest::UpdateInstance(_) => todo!(),
        FHIRRequest::ConditionalUpdate(_) => todo!(),
        FHIRRequest::Patch(_) => todo!(),
        FHIRRequest::DeleteInstance(_) => todo!(),
        FHIRRequest::DeleteType(_) => todo!(),
        FHIRRequest::DeleteSystem(_) => todo!(),
        FHIRRequest::Capabilities => todo!(),
        FHIRRequest::SearchType(_) => todo!(),
        FHIRRequest::SearchSystem(_) => todo!(),
        FHIRRequest::HistoryInstance(_) => todo!(),
        FHIRRequest::HistoryType(_) => todo!(),
        FHIRRequest::HistorySystem(_) => todo!(),
        FHIRRequest::InvokeInstance(_) => todo!(),
        FHIRRequest::InvokeType(_) => todo!(),
        FHIRRequest::InvokeSystem(_) => todo!(),
        FHIRRequest::Batch(_) => todo!(),
        FHIRRequest::Transaction(_) => todo!(),
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
        _ctx: CTX,
        request: crate::request::FHIRRequest,
    ) -> Result<crate::request::FHIRResponse, OperationOutcomeError> {
        let response = self
            .middleware
            .call(self.state.clone(), _ctx, request)
            .await?;

        response
            .response
            .ok_or_else(|| FHIRHTTPError::NoResponse.into())
    }

    async fn capabilities(&self, _ctx: CTX) -> CapabilityStatement {
        todo!()
    }

    async fn search_system(
        &self,
        _ctx: CTX,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn search_type(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn create(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn update(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _id: String,
        _resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn conditional_update(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
        _resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn patch(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _id: String,
        _patches: json_patch::Patch,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
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
            Some(FHIRResponse::Read(read_response)) => Ok(Some(read_response.resource)),
            _ => Err(FHIRHTTPError::NoResponse.into()),
        }
    }

    async fn vread(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _id: String,
        _version_id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _id: String,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_type(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_system(
        &self,
        _ctx: CTX,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn history_system(
        &self,
        _ctx: CTX,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_type(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_instance(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _id: String,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _id: String,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        _ctx: CTX,
        _resource_type: ResourceType,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        _ctx: CTX,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn transaction(
        &self,
        _ctx: CTX,
        _bundle: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn batch(&self, _ctx: CTX, _bundle: Resource) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     use oxidized_fhir_model::r4::generated::resources::ResourceType;

//     use super::*;

//     #[tokio::test]
//     async fn test_fhir_http_client() {
//         let client: FHIRHttpClient<()> =
//             FHIRHttpClient::new(FHIRHttpState::new("https://hapi.fhir.org/baseR4").unwrap());

//         let read_response = client
//             .read((), ResourceType::Patient, "48426182".to_string())
//             .await
//             .unwrap();

//         assert_eq!(
//             Some("48426182".to_string()),
//             read_response.as_ref().map(|r| match r {
//                 Resource::Patient(p) => p.id.as_ref().unwrap().clone(),
//                 _ => panic!("Expected Patient resource"),
//             })
//         );

//         println!("Read response: {:?}", read_response);
//     }
// }
