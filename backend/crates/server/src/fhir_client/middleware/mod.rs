use crate::fhir_client::{ClientState, ServerCTX};
use haste_fhir_client::{
    middleware::{Context, MiddlewareOutput, Next},
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub mod auth_z;
pub mod capabilities;
pub mod check_project;
pub mod custom_models;
pub mod operations;
pub mod rate_limit;
pub mod set_artifact_tenant;
pub mod storage;
pub mod transaction;
pub mod validation;

pub type ServerMiddlewareState<Repository, Search, Terminology> =
    Arc<ClientState<Repository, Search, Terminology>>;
pub type ServerMiddlewareContext<Client> =
    Context<Arc<ServerCTX<Client>>, FHIRRequest, FHIRResponse>;
pub type ServerMiddlewareNext<Client, State> =
    Next<State, ServerMiddlewareContext<Client>, OperationOutcomeError>;
pub type ServerMiddlewareOutput<Client> =
    MiddlewareOutput<ServerMiddlewareContext<Client>, OperationOutcomeError>;
