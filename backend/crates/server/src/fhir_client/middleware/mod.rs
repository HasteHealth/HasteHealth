use crate::fhir_client::{ClientState, ServerCTX};

use oxidized_fhir_client::{
    middleware::{Context, MiddlewareOutput, Next},
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

mod capabilities;
mod set_artifact_tenant;
mod storage;

pub use capabilities::*;
pub use set_artifact_tenant::*;
pub use storage::*;

pub type ServerMiddlewareState<Repository, Search> = Arc<ClientState<Repository, Search>>;
pub type ServerMiddlewareContext<Repo, Search> =
    Context<Arc<ServerCTX<Repo, Search>>, FHIRRequest, FHIRResponse>;
pub type ServerMiddlewareNext<Repo, Search> = Next<
    Arc<ClientState<Repo, Search>>,
    ServerMiddlewareContext<Repo, Search>,
    OperationOutcomeError,
>;
pub type ServerMiddlewareOutput<Repo, Search> =
    MiddlewareOutput<ServerMiddlewareContext<Repo, Search>, OperationOutcomeError>;
