use crate::{
    ServerEnvironmentVariables,
    fhir_client::{
        ServerCTX,
        middleware::{ServerMiddlewareState, operations::ServerOperationContext},
    },
    route_path::api_v1_oidc_path,
};
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRInvokeInstanceRequest, InvocationRequest},
};
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
use std::sync::Arc;
use url::Url;

pub fn idp_registration_info_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    HasteHealthIdpRegistrationInfo::Input,
    HasteHealthIdpRegistrationInfo::Output,
> {
    OperationExecutor::new(
        HasteHealthIdpRegistrationInfo::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
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
