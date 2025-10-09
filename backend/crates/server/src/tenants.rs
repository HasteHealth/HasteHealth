use crate::{
    fhir_client::{FHIRServerClient, ServerCTX},
    services::AppState,
};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType},
    terminology::IssueType,
    types::FHIRString,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{
        ProjectId, TenantId,
        tenant::{CreateTenant, Tenant},
    },
    utilities::generate_id,
};
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}

pub async fn create_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: Arc<AppState<Repo, Search, Terminology>>,
    id: Option<String>,
    _name: &str,
    subscription_tier: &SubscriptionTier,
) -> Result<(), OperationOutcomeError> {
    let transaction_repo = Arc::new(services.repo.transaction().await?);
    {
        let fhir_client = FHIRServerClient::new(
            transaction_repo.clone(),
            services.search.clone(),
            services.terminology.clone(),
        );

        let subscription_tier = serde_json::to_string(subscription_tier).map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Failed to serialize subscription tier".to_string(),
            )
        })?;

        let new_tenant = TenantAuthAdmin::create(
            &*transaction_repo.clone(),
            &TenantId::System,
            CreateTenant {
                id: Some(TenantId::new(id.unwrap_or(generate_id(Some(16))))),
                subscription_tier: Some(subscription_tier),
            },
        )
        .await?;

        let system_project = fhir_client
            .create(
                Arc::new(ServerCTX::root(
                    TenantId::new(new_tenant.id),
                    ProjectId::System,
                )),
                ResourceType::Project,
                Resource::Project(Project {
                    id: Some(ProjectId::System.to_string()),
                    name: Some(Box::new(ProjectId::System.to_string())),
                    ..Default::default()
                }),
            )
            .await?;
    }

    Arc::try_unwrap(transaction_repo)
        .map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Failed to unwrap transaction client".to_string(),
            )
        })?
        .commit()
        .await?;

    Ok(())
}
