use haste_fhir_model::r4::generated::resources::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use haste_jwt::VersionId;
use json_patch::Patch;
use thiserror::Error;

use crate::url::ParsedParameters;

#[derive(Debug, Clone)]
pub struct FHIRCreateRequest {
    pub resource_type: ResourceType,
    pub resource: Resource,
}

#[derive(Debug, Clone)]
pub struct FHIRReadRequest {
    pub resource_type: ResourceType,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct FHIRVersionReadRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub version_id: VersionId,
}

#[derive(Debug, Clone)]
pub struct FHIRUpdateInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub resource: Resource,
}

#[derive(Debug, Clone)]
pub struct FHIRConditionalUpdateRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
    pub resource: Resource,
}

#[derive(Debug, Clone)]
pub struct FHIRPatchRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub patch: Patch,
}

#[derive(Debug, Clone)]
pub struct FHIRHistoryInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub parameters: ParsedParameters,
}

#[derive(Debug, Clone)]
pub struct FHIRHistoryTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Debug, Clone)]
pub struct FHIRHistorySystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Debug, Clone)]
pub struct FHIRDeleteInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct FHIRDeleteTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Debug, Clone)]
pub struct FHIRDeleteSystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Debug, Clone)]
pub struct FHIRSearchTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Debug, Clone)]
pub struct FHIRSearchSystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Error, Debug)]
pub enum OperationParseError {
    #[error("Invalid operation name")]
    Invalid,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct FHIRInvokeInstanceRequest {
    pub operation: Operation,
    pub resource_type: ResourceType,
    pub id: String,
    pub parameters: Parameters,
}

#[derive(Debug, Clone)]
pub struct FHIRInvokeTypeRequest {
    pub operation: Operation,
    pub resource_type: ResourceType,
    pub parameters: Parameters,
}

#[derive(Debug, Clone)]
pub struct FHIRInvokeSystemRequest {
    pub operation: Operation,
    pub parameters: Parameters,
}

#[derive(Debug, Clone)]
pub struct FHIRBatchRequest {
    pub resource: Bundle,
}

#[derive(Debug, Clone)]
pub struct FHIRTransactionRequest {
    pub resource: Bundle,
}

#[derive(Debug, Clone)]
pub enum InvocationRequest {
    Instance(FHIRInvokeInstanceRequest),
    Type(FHIRInvokeTypeRequest),
    System(FHIRInvokeSystemRequest),
}

#[derive(Debug, Clone)]
pub enum HistoryRequest {
    Instance(FHIRHistoryInstanceRequest),
    Type(FHIRHistoryTypeRequest),
    System(FHIRHistorySystemRequest),
}

#[derive(Debug, Clone)]
pub enum SearchRequest {
    Type(FHIRSearchTypeRequest),
    System(FHIRSearchSystemRequest),
}

#[derive(Debug, Clone)]
pub enum DeleteRequest {
    Instance(FHIRDeleteInstanceRequest),
    Type(FHIRDeleteTypeRequest),
    System(FHIRDeleteSystemRequest),
}

#[derive(Debug, Clone)]
pub enum UpdateRequest {
    Instance(FHIRUpdateInstanceRequest),
    Conditional(FHIRConditionalUpdateRequest),
}

#[derive(Debug, Clone)]
pub struct CompartmentRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub request: Box<FHIRRequest>,
}

#[derive(Debug, Clone)]
pub enum FHIRRequest {
    Create(FHIRCreateRequest),

    Read(FHIRReadRequest),
    VersionRead(FHIRVersionReadRequest),

    Update(UpdateRequest),

    Patch(FHIRPatchRequest),

    Delete(DeleteRequest),

    Capabilities,

    Search(SearchRequest),

    History(HistoryRequest),

    Invocation(InvocationRequest),

    Batch(FHIRBatchRequest),
    Transaction(FHIRTransactionRequest),

    Compartment(CompartmentRequest),
}
#[derive(Debug, Clone)]
pub struct FHIRCreateResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRReadResponse {
    pub resource: Option<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRVersionReadResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRUpdateResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRPatchResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRDeleteInstanceResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRDeleteTypeResponse {
    pub resource: Vec<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRDeleteSystemResponse {
    pub resource: Vec<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRCapabilitiesResponse {
    pub capabilities: CapabilityStatement,
}

#[derive(Debug, Clone)]
pub struct FHIRSearchTypeResponse {
    pub bundle: Bundle,
}
#[derive(Debug, Clone)]
pub struct FHIRSearchSystemResponse {
    pub bundle: Bundle,
}
#[derive(Debug, Clone)]
pub struct FHIRHistoryInstanceResponse {
    pub bundle: Bundle,
}
#[derive(Debug, Clone)]
pub struct FHIRHistoryTypeResponse {
    pub bundle: Bundle,
}
#[derive(Debug, Clone)]
pub struct FHIRHistorySystemResponse {
    pub bundle: Bundle,
}
#[derive(Debug, Clone)]
pub struct FHIRInvokeInstanceResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRInvokeTypeResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRInvokeSystemResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRBatchResponse {
    pub resource: Bundle,
}
#[derive(Debug, Clone)]
pub struct FHIRTransactionResponse {
    pub resource: Bundle,
}

#[derive(Debug, Clone)]
pub enum HistoryResponse {
    Instance(FHIRHistoryInstanceResponse),
    Type(FHIRHistoryTypeResponse),
    System(FHIRHistorySystemResponse),
}

#[derive(Debug, Clone)]
pub enum SearchResponse {
    Type(FHIRSearchTypeResponse),
    System(FHIRSearchSystemResponse),
}

#[derive(Debug, Clone)]
pub enum DeleteResponse {
    Instance(FHIRDeleteInstanceResponse),
    Type(FHIRDeleteTypeResponse),
    System(FHIRDeleteSystemResponse),
}

#[derive(Debug, Clone)]
pub enum InvokeResponse {
    Instance(FHIRInvokeInstanceResponse),
    Type(FHIRInvokeTypeResponse),
    System(FHIRInvokeSystemResponse),
}

#[derive(Debug, Clone)]
pub enum FHIRResponse {
    Create(FHIRCreateResponse),

    Read(FHIRReadResponse),
    VersionRead(FHIRVersionReadResponse),

    Update(FHIRUpdateResponse),

    Patch(FHIRPatchResponse),

    Delete(DeleteResponse),

    Capabilities(FHIRCapabilitiesResponse),

    Search(SearchResponse),

    History(HistoryResponse),

    Invoke(InvokeResponse),

    Batch(FHIRBatchResponse),
    Transaction(FHIRTransactionResponse),
}
