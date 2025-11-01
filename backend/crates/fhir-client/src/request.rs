use json_patch::Patch;
use oxidized_fhir_model::r4::generated::resources::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use thiserror::Error;

use crate::ParsedParameter;

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
    pub version_id: String,
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
    pub parameters: Vec<ParsedParameter>,
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
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug, Clone)]
pub struct FHIRHistoryTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug, Clone)]
pub struct FHIRHistorySystemRequest {
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug, Clone)]
pub struct FHIRDeleteInstanceRequest {
    pub resource_type: ResourceType,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct FHIRDeleteTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug, Clone)]
pub struct FHIRDeleteSystemRequest {
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug, Clone)]
pub struct FHIRSearchTypeRequest {
    pub resource_type: ResourceType,
    pub parameters: Vec<ParsedParameter>,
}

#[derive(Debug, Clone)]
pub struct FHIRSearchSystemRequest {
    pub parameters: Vec<ParsedParameter>,
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
#[derive(Debug, Clone)]
pub struct FHIRCreateResponse {
    pub resource: Resource,
}
#[derive(Debug, Clone)]
pub struct FHIRReadResponse {
    pub resource: Resource,
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
    pub total: Option<i64>,
    pub resources: Vec<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRSearchSystemResponse {
    pub total: Option<i64>,
    pub resources: Vec<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRHistoryInstanceResponse {
    pub resources: Vec<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRHistoryTypeResponse {
    pub resources: Vec<Resource>,
}
#[derive(Debug, Clone)]
pub struct FHIRHistorySystemResponse {
    pub resources: Vec<Resource>,
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
