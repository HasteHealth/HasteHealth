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
mod storage;
mod validate_resourcetypes;

pub use capabilities::*;
pub use membership::*;
pub use operations::*;
pub use set_artifact_tenant::*;
pub use storage::*;
pub use validate_resourcetypes::*;

pub type ServerMiddlewareState<Repository, Search, Terminology> =
    Arc<ClientState<Repository, Search, Terminology>>;
pub type ServerMiddlewareContext = Context<Arc<ServerCTX>, FHIRRequest, FHIRResponse>;
pub type ServerMiddlewareNext<Repo, Search, Terminology> = Next<
    Arc<ClientState<Repo, Search, Terminology>>,
    ServerMiddlewareContext,
    OperationOutcomeError,
>;
pub type ServerMiddlewareOutput = MiddlewareOutput<ServerMiddlewareContext, OperationOutcomeError>;
