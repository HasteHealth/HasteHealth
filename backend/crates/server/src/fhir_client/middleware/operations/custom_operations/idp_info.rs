use crate::{
    fhir_client::middleware::{Backend, operations::ServerOperationContext},
    route_path::api_v1_oidc_path,
};
use haste_fhir_client::request::{FHIRInvokeInstanceRequest, InvocationRequest};
use haste_fhir_generated_ops::generated::HasteHealthIdpRegistrationInfo;
use haste_fhir_model::r4::generated::{
    resources::ResourceType, terminology::IssueType, types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, TenantId};
use url::Url;

pub fn idp_registration_info_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    HasteHealthIdpRegistrationInfo::Input,
    HasteHealthIdpRegistrationInfo::Output,
> {
    OperationExecutor::new(
        &HasteHealthIdpRegistrationInfo::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
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
                            IssueType::exception(),
                            "Invalid invocation request type".to_string(),
                        ))
                    });
                };

                if resource_type != &ResourceType::IdentityProvider {
                    return Box::pin(async move {
                        Err(OperationOutcomeError::error(
                            IssueType::invalid(),
                            "Resource type must be IdentityProvider".to_string(),
                        ))
                    });
                }

                let id = id.clone();

                Box::pin(async move {
                    let api_url_string = &context.state.config.api_uri;

                    let api_url = Url::parse(api_url_string).map_err(|e| {
                        tracing::error!("Failed to parse API URL: {:?}", e);
                        OperationOutcomeError::error(
                            IssueType::invalid(),
                            "Invalid API URL configured".to_string(),
                        )
                    })?;

                    let mut idp_callback_path = api_v1_oidc_path(&tenant, &ProjectId::System);
                    idp_callback_path.extend(["federated", &id, "callback"]);

                    let idp_callback_url = api_url
                        .join(
                            idp_callback_path
                                .to_str()
                                .expect("failed to generate idp callback path"),
                        )
                        .map_err(|e| {
                            tracing::error!("Failed to derive FHIR URL: {:?}", e);
                            OperationOutcomeError::error(
                                IssueType::invalid(),
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
