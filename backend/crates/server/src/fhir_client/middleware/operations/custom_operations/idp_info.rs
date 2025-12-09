use crate::{
    ServerEnvironmentVariables, fhir_client::middleware::operations::ServerOperationContext,
};
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
use url::Url;

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
            |context: ServerOperationContext<Repo, Search, Terminology>,
             tenant: TenantId,
             _project: ProjectId,
             request: &InvocationRequest,
             _input: HasteHealthIdpRegistrationInfo::Input| {
                let InvocationRequest::Instance(FHIRInvokeInstanceRequest {
                    resource_type,
                    id,
                    ..
                }) = request
                else {
                    return Box::pin(async move {
                        Err(OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "Invalid invocation request type".to_string(),
                        ))
                    });
                };

                if resource_type != &ResourceType::IdentityProvider {
                    return Box::pin(async move {
                        Err(OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Resource type must be IdentityProvider".to_string(),
                        ))
                    });
                }

                let id = id.clone();

                Box::pin(async move {
                    let api_url_string = context
                        .state
                        .config
                        .get(ServerEnvironmentVariables::APIURI)?;

                    let api_url = Url::parse(&api_url_string).map_err(|e| {
                        tracing::error!("Failed to parse API URL: {:?}", e);
                        OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Invalid API URL configured".to_string(),
                        )
                    })?;

                    let idp_callback_url = api_url
                        .join(
                            format!(
                                "/w/{}/system/api/v1/oidc/federated/{}/callback",
                                tenant.as_ref(),
                                id
                            )
                            .as_str(),
                        )
                        .map_err(|e| {
                            tracing::error!("Failed to derive FHIR URL: {:?}", e);
                            OperationOutcomeError::error(
                                IssueType::Invalid(None),
                                "Invalid API URL configured".to_string(),
                            )
                        })?
                        .to_string();

                    Ok(HasteHealthIdpRegistrationInfo::Output {
                        information: Some(vec![
                            HasteHealthIdpRegistrationInfo::OutputInformation {
                                name: FHIRString {
                                    value: Some("Redirect URL".to_string()),
                                    ..Default::default()
                                },
                                value: FHIRString {
                                    value: Some(idp_callback_url),
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
