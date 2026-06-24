use derivative::Derivative;
use haste_fhir_model::r4::generated::resources::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use haste_jwt::VersionId;
use json_patch::Patch;
use thiserror::Error;

use crate::url::ParsedParameters;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRCreateRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRReadRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRVersionReadRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub version_id: VersionId,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRUpdateInstanceRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRConditionalUpdateRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRPatchRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub patch: Patch,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRHistoryInstanceRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
    pub parameters: ParsedParameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRHistoryTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRHistorySystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRDeleteInstanceRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRDeleteTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRDeleteSystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRSearchTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRSearchSystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Error, Debug)]
pub enum OperationParseError {
    #[error("Invalid operation name")]
    Invalid,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Operation(String);
impl Operation {
    pub fn new(name: &str) -> Result<Self, OperationParseError> {
        let operation_name = name.trim_start_matches('$');
        Ok(Operation(operation_name.to_string()))
    }
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRInvokeInstanceRequest {
    pub operation: Operation,
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub parameters: Parameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRInvokeTypeRequest {
    pub operation: Operation,
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub parameters: Parameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRInvokeSystemRequest {
    pub operation: Operation,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub parameters: Parameters,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRBatchRequest {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Bundle,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRTransactionRequest {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Bundle,
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum InvocationRequest {
    #[derivative(Debug = "transparent")]
    Instance(FHIRInvokeInstanceRequest),
    #[derivative(Debug = "transparent")]
    Type(FHIRInvokeTypeRequest),
    #[derivative(Debug = "transparent")]
    System(FHIRInvokeSystemRequest),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum HistoryRequest {
    #[derivative(Debug = "transparent")]
    Instance(FHIRHistoryInstanceRequest),
    #[derivative(Debug = "transparent")]
    Type(FHIRHistoryTypeRequest),
    #[derivative(Debug = "transparent")]
    System(FHIRHistorySystemRequest),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum SearchRequest {
    #[derivative(Debug = "transparent")]
    Type(FHIRSearchTypeRequest),
    #[derivative(Debug = "transparent")]
    System(FHIRSearchSystemRequest),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum DeleteRequest {
    #[derivative(Debug = "transparent")]
    Instance(FHIRDeleteInstanceRequest),
    #[derivative(Debug = "transparent")]
    Type(FHIRDeleteTypeRequest),
    #[derivative(Debug = "transparent")]
    System(FHIRDeleteSystemRequest),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum UpdateRequest {
    #[derivative(Debug = "transparent")]
    Instance(FHIRUpdateInstanceRequest),
    #[derivative(Debug = "transparent")]
    Conditional(FHIRConditionalUpdateRequest),
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct CompartmentRequest {
    pub resource_type: ResourceType,
    #[derivative(Debug(format_with = "crate::redact"))]
    pub id: String,
    pub request: Box<FHIRRequest>,
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum FHIRRequest {
    #[derivative(Debug = "transparent")]
    Create(FHIRCreateRequest),
    #[derivative(Debug = "transparent")]
    Read(FHIRReadRequest),
    #[derivative(Debug = "transparent")]
    VersionRead(FHIRVersionReadRequest),
    #[derivative(Debug = "transparent")]
    Update(UpdateRequest),
    #[derivative(Debug = "transparent")]
    Patch(FHIRPatchRequest),
    #[derivative(Debug = "transparent")]
    Delete(DeleteRequest),
    Capabilities,
    #[derivative(Debug = "transparent")]
    Search(SearchRequest),
    #[derivative(Debug = "transparent")]
    History(HistoryRequest),
    #[derivative(Debug = "transparent")]
    Invocation(InvocationRequest),
    #[derivative(Debug = "transparent")]
    Batch(FHIRBatchRequest),
    #[derivative(Debug = "transparent")]
    Transaction(FHIRTransactionRequest),
    #[derivative(Debug = "transparent")]
    Compartment(CompartmentRequest),
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRCreateResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRReadResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Option<Resource>,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRVersionReadResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRUpdateResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRPatchResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRDeleteInstanceResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRDeleteTypeResponse {}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRDeleteSystemResponse {}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRCapabilitiesResponse {
    pub capabilities: CapabilityStatement,
}

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRSearchTypeResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub bundle: Bundle,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRSearchSystemResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub bundle: Bundle,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRHistoryInstanceResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub bundle: Bundle,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRHistoryTypeResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub bundle: Bundle,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRHistorySystemResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub bundle: Bundle,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRInvokeInstanceResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRInvokeTypeResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRInvokeSystemResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Resource,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRBatchResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Bundle,
}
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct FHIRTransactionResponse {
    #[derivative(Debug(format_with = "crate::redact"))]
    pub resource: Bundle,
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum HistoryResponse {
    #[derivative(Debug = "transparent")]
    Instance(FHIRHistoryInstanceResponse),
    #[derivative(Debug = "transparent")]
    Type(FHIRHistoryTypeResponse),
    #[derivative(Debug = "transparent")]
    System(FHIRHistorySystemResponse),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum SearchResponse {
    #[derivative(Debug = "transparent")]
    Type(FHIRSearchTypeResponse),
    #[derivative(Debug = "transparent")]
    System(FHIRSearchSystemResponse),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum DeleteResponse {
    #[derivative(Debug = "transparent")]
    Instance(FHIRDeleteInstanceResponse),
    #[derivative(Debug = "transparent")]
    Type(FHIRDeleteTypeResponse),
    #[derivative(Debug = "transparent")]
    System(FHIRDeleteSystemResponse),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum InvokeResponse {
    #[derivative(Debug = "transparent")]
    Instance(FHIRInvokeInstanceResponse),
    #[derivative(Debug = "transparent")]
    Type(FHIRInvokeTypeResponse),
    #[derivative(Debug = "transparent")]
    System(FHIRInvokeSystemResponse),
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum FHIRResponse {
    #[derivative(Debug = "transparent")]
    Create(FHIRCreateResponse),
    #[derivative(Debug = "transparent")]
    Read(FHIRReadResponse),
    #[derivative(Debug = "transparent")]
    VersionRead(FHIRVersionReadResponse),
    #[derivative(Debug = "transparent")]
    Update(FHIRUpdateResponse),
    #[derivative(Debug = "transparent")]
    Patch(FHIRPatchResponse),
    #[derivative(Debug = "transparent")]
    Delete(DeleteResponse),
    #[derivative(Debug = "transparent")]
    Capabilities(FHIRCapabilitiesResponse),
    #[derivative(Debug = "transparent")]
    Search(SearchResponse),
    #[derivative(Debug = "transparent")]
    History(HistoryResponse),
    #[derivative(Debug = "transparent")]
    Invoke(InvokeResponse),
    #[derivative(Debug = "transparent")]
    Batch(FHIRBatchResponse),
    #[derivative(Debug = "transparent")]
    Transaction(FHIRTransactionResponse),
}
