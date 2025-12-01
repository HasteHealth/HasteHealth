use haste_fhir_model::r4::generated::resources::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use json_patch::Patch;

use crate::{
    request::{FHIRRequest, FHIRResponse},
    url::ParsedParameters,
};

#[cfg(feature = "axum")]
pub mod axum;
#[cfg(feature = "http")]
pub mod http;
pub mod middleware;
pub mod request;
pub mod url;

pub trait FHIRClient<CTX, Error>: Send + Sync {
    fn request(
        &self,
        ctx: CTX,
        request: FHIRRequest,
    ) -> impl Future<Output = Result<FHIRResponse, Error>> + Send;

    fn capabilities(
        &self,
        ctx: CTX,
    ) -> impl Future<Output = Result<CapabilityStatement, OperationOutcomeError>> + Send;

    fn search_system(
        &self,
        ctx: CTX,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<Vec<Resource>, Error>> + Send;
    fn search_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<Vec<Resource>, Error>> + Send;

    fn create(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        resource: Resource,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn update(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        resource: Resource,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn conditional_update(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: ParsedParameters,
        resource: Resource,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn patch(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        patches: Patch,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn read(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
    ) -> impl Future<Output = Result<Option<Resource>, Error>> + Send;

    fn vread(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        version_id: String,
    ) -> impl Future<Output = Result<Option<Resource>, Error>> + Send;

    fn delete_instance(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn delete_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn delete_system(
        &self,
        ctx: CTX,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    fn history_system(
        &self,
        ctx: CTX,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<Vec<Resource>, Error>> + Send;

    fn history_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<Vec<Resource>, Error>> + Send;

    fn history_instance(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        parameters: ParsedParameters,
    ) -> impl Future<Output = Result<Vec<Resource>, Error>> + Send;

    fn invoke_instance(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        id: String,
        operation: String,
        parameters: Parameters,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn invoke_type(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        operation: String,
        parameters: Parameters,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn invoke_system(
        &self,
        ctx: CTX,
        operation: String,
        parameters: Parameters,
    ) -> impl Future<Output = Result<Resource, Error>> + Send;

    fn transaction(
        &self,
        ctx: CTX,
        bundle: Bundle,
    ) -> impl Future<Output = Result<Bundle, Error>> + Send;

    fn batch(&self, ctx: CTX, bundle: Bundle)
    -> impl Future<Output = Result<Bundle, Error>> + Send;
}
