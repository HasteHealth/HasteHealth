use crate::fhir_client::{ClientState, ServerCTX};

use oxidized_fhir_client::{
    middleware::{Context, MiddlewareOutput, Next},
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

mod capabilities;
mod membership;
mod operations;
mod set_artifact_tenant;
mod set_project;
mod storage;

pub use capabilities::*;
pub use membership::*;
pub use operations::*;
pub use set_artifact_tenant::*;
pub use set_project::*;
pub use storage::*;

pub type ServerMiddlewareState<Repository, Search, Terminology> =
    Arc<ClientState<Repository, Search, Terminology>>;
pub type ServerMiddlewareContext = Context<Arc<ServerCTX>, FHIRRequest, FHIRResponse>;
pub type ServerMiddlewareNext<Repo, Search, Terminology> = Next<
    Arc<ClientState<Repo, Search, Terminology>>,
    ServerMiddlewareContext,
    OperationOutcomeError,
>;
pub type ServerMiddlewareOutput = MiddlewareOutput<ServerMiddlewareContext, OperationOutcomeError>;
