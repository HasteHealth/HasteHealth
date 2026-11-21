use crate::error::ServerError;
use crate::{
    auth_n::{
        middleware::jwt::derive_well_known_openid_configuration_url,
        oidc::routes::discovery::create_oidc_discovery_document,
    },
    fhir_client::middleware::{Backend, operations::ServerOperationContext},
    route_path::{api_fhir_root_url, api_v1_mcp_path, append_path_segments},
};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::TenantEndpointInformation;
use haste_fhir_model::r4::generated::{terminology::IssueType, types::FHIRUri};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, TenantId};
use url::Url;

pub fn endpoint_metadata_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    TenantEndpointInformation::Input,
    TenantEndpointInformation::Output,
> {
    OperationExecutor::new(
        &TenantEndpointInformation::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             _input: TenantEndpointInformation::Input| {
                Box::pin(async move {
                    let api_url_string = &context.state.config.api_uri;

                    let discovery_document = create_oidc_discovery_document(
                        &tenant,
                        &project,
                        api_url_string,
                    )
                    .map_err(|e| {
                        tracing::error!("Failed to create OIDC discovery document: {:?}", e);
                        OperationOutcomeError::error(
                            IssueType::exception(),
                            "failed to create OIDC discovery document".to_string(),
                        )
                    })?;

                    let api_url = Url::parse(api_url_string).map_err(ServerError::from)?;

                    let fhir_url = api_fhir_root_url(api_url_string, &tenant, &project)?;

                    let fhir_meta_url =
                        append_path_segments(&fhir_url, ["metadata"]).ok_or_else(|| {
                            tracing::error!("Failed to derive FHIR Metadata URL");
                            OperationOutcomeError::error(
                                IssueType::invalid(),
                                "Invalid API URL configured".to_string(),
                            )
                        })?;

                    let mcp_endpiont = api_url
                        .join(api_v1_mcp_path(&tenant, &project).to_str().unwrap())
                        .map_err(ServerError::from)?;

                    Ok(TenantEndpointInformation::Output {
                        fhir_r4_base_url: FHIRUri {
                            value: Some(fhir_url.to_string()),
                            ..Default::default()
                        },
                        fhir_r4_capabilities_url: FHIRUri {
                            value: Some(fhir_meta_url.to_string()),
                            ..Default::default()
                        },
                        oidc_discovery_url: FHIRUri {
                            value: Some(
                                derive_well_known_openid_configuration_url(
                                    api_url_string,
                                    &tenant,
                                    &project,
                                )?
                                .to_string(),
                            ),
                            ..Default::default()
                        },
                        oidc_token_endpoint: FHIRUri {
                            value: Some(discovery_document.token_endpoint),
                            ..Default::default()
                        },
                        oidc_authorize_endpoint: FHIRUri {
                            value: Some(discovery_document.authorization_endpoint),
                            ..Default::default()
                        },
                        oidc_jwks_endpoint: FHIRUri {
                            value: Some(discovery_document.jwks_uri),
                            ..Default::default()
                        },
                        mcp_endpoint: FHIRUri {
                            value: Some(mcp_endpiont.to_string()),
                            ..Default::default()
                        },
                    })
                })
            },
        ),
    )
}
