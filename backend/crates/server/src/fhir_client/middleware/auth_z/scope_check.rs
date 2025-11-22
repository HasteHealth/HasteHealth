#![allow(unused)]
use crate::fhir_client::{
    FHIRServerClient, ServerCTX, ServerClientConfig,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_access_control::PolicyContext;
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, Resource},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{
    ProjectId, UserRole,
    scopes::{Scope, SmartResourceScopePermissions},
};
use haste_repository::Repository;
use std::sync::Arc;

// pub fn request_type_to_permission(request: &FHIRRequest) -> Option<SmartResourceScopePermissions> {
//     match request {
//         FHIRRequest::Batch(_)
//         | FHIRRequest::Transaction(_)
//         | FHIRRequest::InvokeInstance(_)
//         | FHIRRequest::InvokeType(_)
//         | FHIRRequest::InvokeSystem(_) => Some(SmartResourceScopePermissions ),
//         FHIRRequest::Create(_) => Some(Scope::Write),
//         FHIRRequest::Read(_) => Some(Scope::Read),
//         FHIRRequest::VersionRead(_) => Some(Scope::Read),
//         FHIRRequest::UpdateInstance(_) => Some(Scope::Write),
//         FHIRRequest::ConditionalUpdate(_) => Some(Scope::Write),
//         FHIRRequest::Patch(_) => Some(Scope::Write),
//         FHIRRequest::DeleteInstance(_) => Some(Scope::Write),
//         FHIRRequest::DeleteType(_) => Some(Scope::SystemWrite),
//         FHIRRequest::DeleteSystem(_) => Some(Scope::SystemWrite),
//         FHIRRequest::Capabilities => Some(Scope::Read),
//         FHIRRequest::SearchType(_) => Some(Scope::Read),
//         FHIRRequest::SearchSystem(_) => Some(Scope::SystemRead),
//         FHIRRequest::HistoryInstance(_) => Some(Scope::Read),
//         FHIRRequest::HistoryType(_) => Some(Scope::Read),
//         FHIRRequest::HistorySystem(_) => Some(Scope::SystemRead),
//     }
// }

pub struct AccessControlMiddleware {}
impl AccessControlMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for AccessControlMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
            match &context.request {
                // Batch and transaction will call back into this middleware for their individual requests
                // at which point the permissions will be checked.
                FHIRRequest::Batch(_)
                | FHIRRequest::Transaction(_)
                | FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_) => {
                    if let Some(next) = next {
                        Ok(next(state, context).await?)
                    } else {
                        Ok(context)
                    }
                }
                FHIRRequest::Create(fhircreate_request) => todo!(),
                FHIRRequest::Read(fhirread_request) => todo!(),
                FHIRRequest::VersionRead(fhirversion_read_request) => todo!(),
                FHIRRequest::UpdateInstance(fhirupdate_instance_request) => todo!(),
                FHIRRequest::ConditionalUpdate(fhirconditional_update_request) => todo!(),
                FHIRRequest::Patch(fhirpatch_request) => todo!(),
                FHIRRequest::DeleteInstance(fhirdelete_instance_request) => todo!(),
                FHIRRequest::DeleteType(fhirdelete_type_request) => todo!(),
                FHIRRequest::DeleteSystem(fhirdelete_system_request) => todo!(),
                FHIRRequest::Capabilities => todo!(),
                FHIRRequest::SearchType(fhirsearch_type_request) => todo!(),
                FHIRRequest::SearchSystem(fhirsearch_system_request) => todo!(),
                FHIRRequest::HistoryInstance(fhirhistory_instance_request) => todo!(),
                FHIRRequest::HistoryType(fhirhistory_type_request) => todo!(),
                FHIRRequest::HistorySystem(fhirhistory_system_request) => todo!(),
            }
        })
    }
}
