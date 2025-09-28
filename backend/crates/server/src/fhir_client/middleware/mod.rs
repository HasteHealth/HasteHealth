use crate::fhir_client::{ClientState, ServerCTX};

use oxidized_fhir_client::{
    middleware::{Context, MiddlewareOutput, Next},
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

mod capabilities;
mod operations;
mod set_artifact_tenant;
mod storage;

pub use capabilities::*;
pub use operations::*;
pub use set_artifact_tenant::*;
pub use storage::*;

pub type ServerMiddlewareState<Repository, Search> = Arc<ClientState<Repository, Search>>;
pub type ServerMiddlewareContext = Context<Arc<ServerCTX>, FHIRRequest, FHIRResponse>;
pub type ServerMiddlewareNext<Repo, Search> =
    Next<Arc<ClientState<Repo, Search>>, ServerMiddlewareContext, OperationOutcomeError>;
pub type ServerMiddlewareOutput = MiddlewareOutput<ServerMiddlewareContext, OperationOutcomeError>;
