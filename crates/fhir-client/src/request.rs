use json_patch::Patch;
use oxidized_fhir_model::r4::types::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use thiserror::Error;

use crate::ParsedParameter;

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
    pub version_id: String,
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
    pub parameters: Vec<ParsedParameter>,
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
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug)]
pub struct FHIRHistoryTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug)]
pub struct FHIRHistorySystemRequest {
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug)]
pub struct FHIRDeleteInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
}

#[derive(Debug)]
pub struct FHIRDeleteTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug)]
pub struct FHIRDeleteSystemRequest {
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug)]
pub struct FHIRSearchTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug)]
pub struct FHIRSearchSystemRequest {
    pub parameters: Vec<ParsedParameter>,
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
pub enum FHIRRequest {
    Create(FHIRCreateRequest),

    Read(FHIRReadRequest),
    VersionRead(FHIRVersionReadRequest),

    UpdateInstance(FHIRUpdateInstanceRequest),
    ConditionalUpdate(FHIRConditionalUpdateRequest),

    Patch(FHIRPatchRequest),

    DeleteInstance(FHIRDeleteInstanceRequest),
    DeleteType(FHIRDeleteTypeRequest),
    DeleteSystem(FHIRDeleteSystemRequest),

    Capabilities,

    SearchType(FHIRSearchTypeRequest),
    SearchSystem(FHIRSearchSystemRequest),

    HistoryInstance(FHIRHistoryInstanceRequest),
    HistoryType(FHIRHistoryTypeRequest),
    HistorySystem(FHIRHistorySystemRequest),

    InvokeInstance(FHIRInvokeInstanceRequest),
    InvokeType(FHIRInvokeTypeRequest),
    InvokeSystem(FHIRInvokeSystemRequest),

    Batch(FHIRBatchRequest),
    Transaction(FHIRTransactionRequest),
}

pub struct FHIRCreateResponse {
    pub resource: Resource,
}

pub struct FHIRReadResponse {
    pub resource: Resource,
}

pub struct FHIRVersionReadResponse {
    pub resource: Resource,
}

pub struct FHIRUpdateResponse {
    pub resource: Resource,
}
pub struct FHIRPatchResponse {
    pub resource: Resource,
}

pub struct FHIRDeleteInstanceResponse {
    pub resource: Resource,
}

pub struct FHIRDeleteTypeResponse {
    pub resource: Vec<Resource>,
}

pub struct FHIRDeleteSystemResponse {
    pub resource: Vec<Resource>,
}

pub struct FHIRCapabilitiesResponse {
    pub capabilities: CapabilityStatement,
}

pub struct FHIRSearchTypeResponse {
    pub resources: Vec<Resource>,
}
pub struct FHIRSearchSystemResponse {
    pub resources: Vec<Resource>,
}

pub struct FHIRHistoryInstanceResponse {
    pub resources: Vec<Resource>,
}
pub struct FHIRHistoryTypeResponse {
    pub resources: Vec<Resource>,
}
pub struct FHIRHistorySystemResponse {
    pub resources: Vec<Resource>,
}

pub struct FHIRInvokeInstanceResponse {
    pub resource: Parameters,
}

pub struct FHIRInvokeTypeResponse {
    pub resource: Parameters,
}

pub struct FHIRInvokeSystemResponse {
    pub resource: Parameters,
}

pub struct FHIRBatchResponse {
    pub resource: Bundle,
}

pub struct FHIRTransactionResponse {
    pub resource: Bundle,
}

pub enum FHIRResponse {
    Create(FHIRCreateResponse),

    Read(FHIRReadResponse),
    VersionRead(FHIRVersionReadResponse),

    Update(FHIRUpdateResponse),

    Patch(FHIRPatchResponse),

    DeleteInstance(FHIRDeleteInstanceResponse),
    DeleteType(FHIRDeleteTypeResponse),
    DeleteSystem(FHIRDeleteSystemResponse),

    Capabilities(FHIRCapabilitiesResponse),

    SearchType(FHIRSearchTypeResponse),
    SearchSystem(FHIRSearchSystemResponse),

    HistoryInstance(FHIRHistoryInstanceResponse),
    HistoryType(FHIRHistoryTypeResponse),
    HistorySystem(FHIRHistorySystemResponse),

    InvokeInstance(FHIRInvokeInstanceResponse),
    InvokeType(FHIRInvokeTypeResponse),
    InvokeSystem(FHIRInvokeSystemResponse),

    Batch(FHIRBatchResponse),
    Transaction(FHIRTransactionResponse),
}
