use std::sync::Arc;

use reqwest::Url;
use thiserror::Error;

use crate::{
    FHIRClient,
    middleware::{Middleware, MiddlewareChain},
    request::{FHIRRequest, FHIRResponse},
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

impl<CTX: 'static + Send + Sync> FHIRHttpClient<CTX> {
    pub fn new(
        state: FHIRHttpState,
        middleware_chain: Vec<MiddlewareChain<Arc<FHIRHttpState>, CTX, FHIRRequest, FHIRResponse>>,
    ) -> Self {
        let middleware = Middleware::new(middleware_chain);
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

    async fn capabilities(&self, ctx: CTX) -> fhir_model::r4::types::CapabilityStatement {
        todo!()
    }

    async fn search_system(
        &self,
        ctx: CTX,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn search_type(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn create(
        &self,
        ctx: CTX,
        resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn update(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
        resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn conditional_update(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        parameters: Vec<crate::ParsedParameter>,
        resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn patch(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
        patches: json_patch::Patch,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn read(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn vread(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
        version_id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
    ) -> Result<(), FHIRHTTPError> {
        todo!()
    }

    async fn delete_type(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<(), FHIRHTTPError> {
        todo!()
    }

    async fn delete_system(
        &self,
        ctx: CTX,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<(), FHIRHTTPError> {
        todo!()
    }

    async fn history_system(
        &self,
        ctx: CTX,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn history_type(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn history_instance(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
        parameters: Vec<crate::ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, FHIRHTTPError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
        operation: String,
        parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        ctx: CTX,
        resource_type: fhir_model::r4::types::ResourceType,
        operation: String,
        parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        ctx: CTX,
        operation: String,
        parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn transaction(
        &self,
        ctx: CTX,
        bundle: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }

    async fn batch(
        &self,
        ctx: CTX,
        bundle: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, FHIRHTTPError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fhir_http_client() {
        let fhir_http: FHIRHttpClient<()> = FHIRHttpClient::new(
            FHIRHttpState::new("http://example.com/fhir").unwrap(),
            vec![],
        );
    }
}
