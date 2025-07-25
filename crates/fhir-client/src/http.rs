use std::{pin::Pin, sync::Arc};

use fhir_model::r4::types::Resource;
use fhir_serialization_json::errors::DeserializeError;
use reqwest::Url;
use thiserror::Error;

use crate::{
    FHIRClient,
    middleware::{Context, Middleware, Next},
    request::{FHIRReadResponse, FHIRRequest, FHIRResponse},
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
    middleware: Middleware<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse>,
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
}

fn fhir_request_to_http_request<CTX>(
    state: &FHIRHttpState,
    request: FHIRRequest,
) -> Result<reqwest::Request, FHIRHTTPError> {
    match request {
        FHIRRequest::Read(read_request) => {
            let read_request_url = state
                .api_url
                .join(&format!(
                    "{}/{}",
                    read_request.resource_type.as_str(),
                    read_request.id
                ))
                .map_err(|_e| FHIRHTTPError::UrlParseError("Read request".to_string()))?;

            let request = state.client.get(read_request_url).build()?;
            Ok(request)
        }
        _ => Err(FHIRHTTPError::NotSupported),
    }
}

async fn http_response_to_fhir_response(
    fhir_request: FHIRRequest,
    response: reqwest::Response,
) -> Result<FHIRResponse, FHIRHTTPError> {
    match fhir_request {
        FHIRRequest::Read(_) => {
            let status = response.status();
            let body = response
                .bytes()
                .await
                .map_err(FHIRHTTPError::RequestError)?;
            let resource = fhir_serialization_json::from_bytes::<Resource>(&body)?;
            Ok(FHIRResponse::Read(FHIRReadResponse { resource }))
        }
        _ => Err(FHIRHTTPError::NotSupported),
    }
}

fn http_middleware<CTX: Send + Sync + 'static>(
    state: Arc<FHIRHttpState>,
    x: Context<CTX, FHIRRequest, FHIRResponse>,
    _next: Option<Arc<Next<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse>>>,
) -> Pin<Box<dyn Future<Output = Context<CTX, FHIRRequest, FHIRResponse>> + Send>> {
    Box::pin(async {
        let mut x = if let Some(next) = _next {
            let p = next(state, x).await;
            p
        } else {
            x
        };

        println!("Middleware 1 executed");
        x
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
    type Middleware = Middleware<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse>;

    async fn request(
        &self,
        _ctx: CTX,
        request: crate::request::FHIRRequest,
    ) -> Result<crate::request::FHIRResponse, FHIRHTTPError> {
        let response = self
            .middleware
            .call(self.state.clone(), _ctx, request)
            .await;
        response.response.ok_or_else(|| FHIRHTTPError::NoResponse)
    }

    fn middleware(&self) -> &Self::Middleware {
        &self.middleware
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
        _ctx: CTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
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
            FHIRHttpClient::new(FHIRHttpState::new("http://example.com/fhir").unwrap());

        let read_response = client
            .read(
                (),
                ResourceType::new("Patient".to_string()).unwrap(),
                "123".to_string(),
            )
            .await
            .unwrap();

        println!("Read response: {:?}", read_response);
    }
}
