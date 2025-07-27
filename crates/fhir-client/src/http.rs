use std::{pin::Pin, sync::Arc};

use fhir_model::r4::types::{OperationOutcome, Resource};
use fhir_serialization_json::errors::DeserializeError;
use reqwest::Url;
use thiserror::Error;

use crate::{
    FHIRClient,
    middleware::{Context, Middleware, Next},
    request::{self, FHIRReadResponse, FHIRRequest, FHIRResponse},
};

pub struct FHIRHttpState {
    client: reqwest::Client,
    api_url: Url,
}

impl FHIRHttpState {
    pub fn new(api_url: &str) -> Result<Self, FHIRHTTPError> {
        let url =
            Url::parse(api_url).map_err(|_| FHIRHTTPError::UrlParseError(api_url.to_string()))?;
        Ok(FHIRHttpState {
            client: reqwest::Client::new(),
            api_url: url,
        })
    }
}

pub struct FHIRHttpClient<CTX> {
    state: Arc<FHIRHttpState>,
    middleware: Middleware<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse, FHIRHTTPError>,
}

#[derive(Error, Debug)]
pub enum FHIRHTTPError {
    #[error("Remote error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Not supported operation")]
    NotSupported,
    #[error("No response received")]
    NoResponse,
    #[error("Failed to parse URL for: '{0}'")]
    UrlParseError(String),
    #[error("Failed to parse FHIR serialization: {0}")]
    FHIRSerializationError(#[from] DeserializeError),
    #[error("Remote error: {0}, request: {1}")]
    RemoteError(reqwest::StatusCode, String),
    #[error("Operation Error")]
    OperationError(OperationOutcome),
}

fn fhir_request_to_http_request(
    state: &FHIRHttpState,
    request: &FHIRRequest,
) -> Result<reqwest::Request, FHIRHTTPError> {
    match request {
        FHIRRequest::Read(read_request) => {
            let read_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}/{}",
                    state.api_url.path(),
                    read_request.resource_type.as_str(),
                    read_request.id
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Read request".to_string()))?;

            let request = state
                .client
                .get(read_request_url)
                .header("Accept", "application/fhir+json")
                .header("Content-Type", "application/fhir+json, application/json")
                .build()?;
            Ok(request)
        }
        _ => Err(FHIRHTTPError::NotSupported),
    }
}

async fn http_response_to_fhir_response(
    fhir_request: &FHIRRequest,
    response: reqwest::Response,
) -> Result<FHIRResponse, FHIRHTTPError> {
    match fhir_request {
        FHIRRequest::Read(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::RequestError)?;

            if !status.is_success() {
                if let Ok(operation_outcome) =
                    fhir_serialization_json::from_bytes::<OperationOutcome>(&body)
                {
                    return Err(FHIRHTTPError::OperationError(operation_outcome));
                }
                return Err(FHIRHTTPError::RemoteError(
                    status,
                    "Failed to read resource".to_string(),
                ));
            }

            let resource = fhir_serialization_json::from_bytes::<Resource>(&body)?;
            Ok(FHIRResponse::Read(FHIRReadResponse { resource }))
        }
        _ => Err(FHIRHTTPError::NotSupported),
    }
}

fn http_middleware<CTX: Send + Sync + 'static>(
    state: Arc<FHIRHttpState>,
    context: Context<CTX, FHIRRequest, FHIRResponse>,
    _next: Option<Arc<Next<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse, FHIRHTTPError>>>,
) -> Pin<
    Box<dyn Future<Output = Result<Context<CTX, FHIRRequest, FHIRResponse>, FHIRHTTPError>> + Send>,
> {
    Box::pin(async move {
        let http_request = fhir_request_to_http_request(&state, &context.request)?;
        let response = state
            .client
            .execute(http_request)
            .await
            .map_err(FHIRHTTPError::RequestError)?;
        let mut next_context = context;
        let fhir_response = http_response_to_fhir_response(&next_context.request, response).await?;
        next_context.response = Some(fhir_response);

        Ok(next_context)
    })
}

impl<CTX: 'static + Send + Sync> FHIRHttpClient<CTX> {
    pub fn new(state: FHIRHttpState) -> Self {
        let middleware = Middleware::new(vec![Box::new(http_middleware)]);
        FHIRHttpClient {
            state: Arc::new(state),
            middleware,
        }
    }
}

impl<CTX: 'static + Send + Sync> FHIRClient<CTX, FHIRHTTPError> for FHIRHttpClient<CTX> {
    async fn request(
        &self,
        _ctx: CTX,
        request: crate::request::FHIRRequest,
    ) -> Result<crate::request::FHIRResponse, FHIRHTTPError> {
        let response = self
            .middleware
            .call(self.state.clone(), _ctx, request)
            .await?;

        response.response.ok_or_else(|| FHIRHTTPError::NoResponse)
    }

    async fn capabilities(&self, _ctx: CTX) -> fhir_model::r4::types::CapabilityStatement {
        todo!()
    }

    async fn search_system(
        &self,
        _ctx: CTX,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn search_type(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn create(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn update(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn conditional_update(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
        _resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn patch(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _patches: json_patch::Patch,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn read(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, FHIRHTTPError> {
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
            _ => Err(FHIRHTTPError::NoResponse),
        }
    }

    async fn vread(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _version_id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
    ) -> Result<(), FHIRHTTPError> {
        todo!()
    }

    async fn delete_type(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<(), FHIRHTTPError> {
        todo!()
    }

    async fn delete_system(
        &self,
        _ctx: CTX,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<(), FHIRHTTPError> {
        todo!()
    }

    async fn history_system(
        &self,
        _ctx: CTX,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn history_type(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn history_instance(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _operation: String,
        _parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _operation: String,
        _parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        _ctx: CTX,
        _operation: String,
        _parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn transaction(
        &self,
        _ctx: CTX,
        _bundle: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn batch(
        &self,
        _ctx: CTX,
        _bundle: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use fhir_model::r4::types::ResourceType;

    use super::*;

    #[tokio::test]
    async fn test_fhir_http_client() {
        let client: FHIRHttpClient<()> =
            FHIRHttpClient::new(FHIRHttpState::new("https://hapi.fhir.org/baseR4").unwrap());

        let read_response = client
            .read(
                (),
                ResourceType::new("Patient".to_string()).unwrap(),
                "48426182".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(
            Some("48426182".to_string()),
            read_response.as_ref().map(|r| match r {
                Resource::Patient(p) => p.id.as_ref().unwrap().clone(),
                _ => panic!("Expected Patient resource"),
            })
        );

        println!("Read response: {:?}", read_response);
    }
}
