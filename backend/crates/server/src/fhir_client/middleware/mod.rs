#![allow(unused)]
use crate::fhir_client::{ClientState, ServerCTX};

use oxidized_fhir_client::{
    middleware::{Context, MiddlewareOutput, Next},
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub mod access_control;
pub mod capabilities;
pub mod check_project;
pub mod custom_models;
pub mod operations;
pub mod set_artifact_tenant;
pub mod storage;
pub mod transaction;

pub type ServerMiddlewareState<Repository, Search, Terminology> =
    Arc<ClientState<Repository, Search, Terminology>>;
pub type ServerMiddlewareContext = Context<Arc<ServerCTX>, FHIRRequest, FHIRResponse>;
pub type ServerMiddlewareNext<Repo, Search, Terminology> = Next<
    Arc<ClientState<Repo, Search, Terminology>>,
    ServerMiddlewareContext,
    OperationOutcomeError,
>;
pub type ServerMiddlewareOutput = MiddlewareOutput<ServerMiddlewareContext, OperationOutcomeError>;
