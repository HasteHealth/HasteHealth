use haste_fhir_model::r4::generated::resources::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use haste_jwt::VersionId;
use json_patch::Patch;
use thiserror::Error;

use crate::url::ParsedParameters;

#[derive(Debug)]
pub struct FHIRCreateRequest {
    pub resource_type: ResourceType,
    pub resource: Resource,
}

#[derive(Debug)]
pub struct FHIRReadRequest {
    pub resource_type: ResourceType,
    pub id: String,
}

#[derive(Debug)]
pub struct FHIRVersionReadRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub version_id: VersionId,
}

#[derive(Debug)]
pub struct FHIRUpdateInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub resource: Resource,
}

#[derive(Debug)]
pub struct FHIRConditionalUpdateRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
    pub resource: Resource,
}

#[derive(Debug)]
pub struct FHIRPatchRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub patch: Patch,
}

#[derive(Debug)]
pub struct FHIRHistoryInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
    pub parameters: ParsedParameters,
}

#[derive(Debug)]
pub struct FHIRHistoryTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Debug)]
pub struct FHIRHistorySystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Debug)]
pub struct FHIRDeleteInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
}

#[derive(Debug)]
pub struct FHIRDeleteTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Debug)]
pub struct FHIRDeleteSystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Debug)]
pub struct FHIRSearchTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: ParsedParameters,
}

#[derive(Debug)]
pub struct FHIRSearchSystemRequest {
    pub parameters: ParsedParameters,
}

#[derive(Error, Debug)]
pub enum OperationParseError {
    #[error("Invalid operation name")]
    Invalid,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct FHIRInvokeInstanceRequest {
    pub operation: Operation,
    pub resource_type: ResourceType,
    pub id: String,
    pub parameters: Parameters,
}

#[derive(Debug)]
pub struct FHIRInvokeTypeRequest {
    pub operation: Operation,
    pub resource_type: ResourceType,
    pub parameters: Parameters,
}

#[derive(Debug)]
pub struct FHIRInvokeSystemRequest {
    pub operation: Operation,
    pub parameters: Parameters,
}

#[derive(Debug)]
pub struct FHIRBatchRequest {
    pub resource: Bundle,
}

#[derive(Debug)]
pub struct FHIRTransactionRequest {
    pub resource: Bundle,
}

#[derive(Debug)]
pub enum InvocationRequest {
    Instance(FHIRInvokeInstanceRequest),
    Type(FHIRInvokeTypeRequest),
    System(FHIRInvokeSystemRequest),
}

#[derive(Debug)]
pub enum HistoryRequest {
    Instance(FHIRHistoryInstanceRequest),
    Type(FHIRHistoryTypeRequest),
    System(FHIRHistorySystemRequest),
}

#[derive(Debug)]
pub enum SearchRequest {
    Type(FHIRSearchTypeRequest),
    System(FHIRSearchSystemRequest),
}

#[derive(Debug)]
pub enum DeleteRequest {
    Instance(FHIRDeleteInstanceRequest),
    Type(FHIRDeleteTypeRequest),
    System(FHIRDeleteSystemRequest),
}

#[derive(Debug)]
pub enum UpdateRequest {
    Instance(FHIRUpdateInstanceRequest),
    Conditional(FHIRConditionalUpdateRequest),
}

#[derive(Debug)]
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
}
#[derive(Debug)]
pub struct FHIRCreateResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRReadResponse {
    pub resource: Option<Resource>,
}
#[derive(Debug)]
pub struct FHIRVersionReadResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRUpdateResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRPatchResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRDeleteInstanceResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRDeleteTypeResponse {
    pub resource: Vec<Resource>,
}
#[derive(Debug)]
pub struct FHIRDeleteSystemResponse {
    pub resource: Vec<Resource>,
}
#[derive(Debug)]
pub struct FHIRCapabilitiesResponse {
    pub capabilities: CapabilityStatement,
}

#[derive(Debug)]
pub struct FHIRSearchTypeResponse {
    pub bundle: Bundle,
}
#[derive(Debug)]
pub struct FHIRSearchSystemResponse {
    pub bundle: Bundle,
}
#[derive(Debug)]
pub struct FHIRHistoryInstanceResponse {
    pub bundle: Bundle,
}
#[derive(Debug)]
pub struct FHIRHistoryTypeResponse {
    pub bundle: Bundle,
}
#[derive(Debug)]
pub struct FHIRHistorySystemResponse {
    pub bundle: Bundle,
}
#[derive(Debug)]
pub struct FHIRInvokeInstanceResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRInvokeTypeResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRInvokeSystemResponse {
    pub resource: Resource,
}
#[derive(Debug)]
pub struct FHIRBatchResponse {
    pub resource: Bundle,
}
#[derive(Debug)]
pub struct FHIRTransactionResponse {
    pub resource: Bundle,
}

#[derive(Debug)]
pub enum HistoryResponse {
    Instance(FHIRHistoryInstanceResponse),
    Type(FHIRHistoryTypeResponse),
    System(FHIRHistorySystemResponse),
}

#[derive(Debug)]
pub enum SearchResponse {
    Type(FHIRSearchTypeResponse),
    System(FHIRSearchSystemResponse),
}

#[derive(Debug)]
pub enum DeleteResponse {
    Instance(FHIRDeleteInstanceResponse),
    Type(FHIRDeleteTypeResponse),
    System(FHIRDeleteSystemResponse),
}

#[derive(Debug)]
pub enum InvokeResponse {
    Instance(FHIRInvokeInstanceResponse),
    Type(FHIRInvokeTypeResponse),
    System(FHIRInvokeSystemResponse),
}

#[derive(Debug)]
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
