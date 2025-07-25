use crate::{
    FHIRClient,
    middleware::{Middleware, MiddlewareChain},
    request::{FHIRRequest, FHIRResponse},
};

pub struct FHIRHttpClient<CTX> {
    middleware: Middleware<(), CTX, FHIRRequest, FHIRResponse>,
}

impl<CTX: 'static + Send + Sync> FHIRHttpClient<CTX> {
    pub fn new(middleware_chain: Vec<MiddlewareChain<(), CTX, FHIRRequest, FHIRResponse>>) -> Self {
        let middleware = Middleware::new(middleware_chain);
        FHIRHttpClient { middleware }
    }
}

pub enum FHIRHTTPError {
    NoResponse,
}

impl<CTX: 'static + Send + Sync> FHIRClient<CTX, FHIRHTTPError> for FHIRHttpClient<CTX> {
    type Middleware = Middleware<(), CTX, FHIRRequest, FHIRResponse>;

    async fn request(
        &self,
        _ctx: CTX,
        request: crate::request::FHIRRequest,
    ) -> Result<crate::request::FHIRResponse, FHIRHTTPError> {
        let response = self.middleware.call((), _ctx, request).await;
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
