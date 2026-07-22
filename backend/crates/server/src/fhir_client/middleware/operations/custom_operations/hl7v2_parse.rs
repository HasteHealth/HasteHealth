use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::Hl7v2Parse;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use std::sync::Arc;

pub fn hl7v2_parse_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    Hl7v2Parse::Input,
    Hl7v2Parse::Output,
> {
    OperationExecutor::new(
        Hl7v2Parse::CODE.to_string(),
        Box::new(
            |_context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             _tenant: TenantId,
             _project: ProjectId,
             _request: &InvocationRequest,
             input: Hl7v2Parse::Input| {
                Box::pin(async move {
                    if let Some(hl7v2_message) = input.hl7v2.value.as_ref() {
                        let message = haste_hl7v2::parser::ParsedHL7V2Message::try_from(
                            hl7v2_message.as_str(),
                        )?;

                        Ok(Hl7v2Parse::Output { hl7v2: message.0 })
                    } else {
                        Err(OperationOutcomeError::error(
                            IssueType::invalid(),
                            "Missing hl7v2 message".to_string(),
                        ))
                    }
                })
            },
        ),
    )
}
