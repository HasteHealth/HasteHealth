use crate::fhir_client::middleware::operations::ServerOperationContext;
use haste_fhir_client::request::{FHIRInvokeInstanceRequest, InvocationRequest};
use haste_fhir_generated_ops::generated::HasteHealthIdpRegistrationInfo;
use haste_fhir_model::r4::generated::{
    resources::ResourceType, terminology::IssueType, types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;

pub fn idp_registration_info<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    HasteHealthIdpRegistrationInfo::Input,
    HasteHealthIdpRegistrationInfo::Output,
> {
    OperationExecutor::new(
        HasteHealthIdpRegistrationInfo::CODE.to_string(),
        Box::new(
            |_context: ServerOperationContext<Repo, Search, Terminology>,
             _tenant: TenantId,
             _project: ProjectId,
             request: &InvocationRequest,
             _input: HasteHealthIdpRegistrationInfo::Input| {
                Box::pin(async move {
                    let InvocationRequest::Instance(FHIRInvokeInstanceRequest {
                        resource_type,
                        id,
                        ..
                    }) = request
                    else {
                        return Err(OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "Invalid invocation request type".to_string(),
                        ));
                    };

                    if resource_type != &ResourceType::IdentityProvider {
                        return Err(OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Resource type must be IdentityProvider".to_string(),
                        ));
                    }

                    Ok(HasteHealthIdpRegistrationInfo::Output {
                        information: Some(vec![
                            HasteHealthIdpRegistrationInfo::OutputInformation {
                                name: FHIRString {
                                    value: Some("Redirect URL".to_string()),
                                    ..Default::default()
                                },
                                value: FHIRString {
                                    ..Default::default()
                                },
                            },
                        ]),
                    })
                })
            },
        ),
    )
}
