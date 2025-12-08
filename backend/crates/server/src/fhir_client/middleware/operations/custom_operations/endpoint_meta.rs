use crate::{
    ServerEnvironmentVariables,
    auth_n::{
        middleware::jwt::derive_well_known_url,
        oidc::routes::discovery::create_oidc_discovery_document,
    },
    fhir_client::middleware::operations::ServerOperationContext,
};
use haste_fhir_generated_ops::generated::TenantEndpointInformation;
use haste_fhir_model::r4::generated::{terminology::IssueType, types::FHIRUri};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use url::Url;

pub fn endpoint_metadata<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    TenantEndpointInformation::Input,
    TenantEndpointInformation::Output,
> {
    OperationExecutor::new(
        TenantEndpointInformation::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<Repo, Search, Terminology>,
             tenant: TenantId,
             project: ProjectId,
             _input: TenantEndpointInformation::Input| {
                Box::pin(async move {
                    let api_url_string = context
                        .state
                        .config
                        .get(ServerEnvironmentVariables::APIURI)?;

                    let discovery_document =
                        create_oidc_discovery_document(&tenant, &project, &api_url_string)
                            .map_err(|e| {
                                tracing::error!(
                                    "Failed to create OIDC discovery document: {:?}",
                                    e
                                );
                                OperationOutcomeError::error(
                                    IssueType::Exception(None),
                                    "failed to create OIDC discovery document".to_string(),
                                )
                            })?;
                    let api_url = Url::parse(&api_url_string).map_err(|e| {
                        tracing::error!("Failed to parse API URL: {:?}", e);
                        OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Invalid API URL configured".to_string(),
                        )
                    })?;

                    let fhir_url = api_url
                        .join(
                            format!("/w/{}/{}/api/v1/fhir/r4", tenant.as_ref(), project.as_ref())
                                .as_str(),
                        )
                        .map_err(|e| {
                            tracing::error!("Failed to derive FHIR URL: {:?}", e);
                            OperationOutcomeError::error(
                                IssueType::Invalid(None),
                                "Invalid API URL configured".to_string(),
                            )
                        })?;

                    let fhir_meta_url = fhir_url.join("metadata").map_err(|e| {
                        tracing::error!("Failed to derive FHIR Metadata URL: {:?}", e);
                        OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Invalid API URL configured".to_string(),
                        )
                    })?;

                    let mcp_endpiont = api_url.join("/w/{}/{}/api/v1/mcp").map_err(|e| {
                        tracing::error!("Failed to derive MCP Endpoint URL: {:?}", e);
                        OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Invalid API URL configured".to_string(),
                        )
                    })?;

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
                                derive_well_known_url(&api_url_string, &tenant, &project)?
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
