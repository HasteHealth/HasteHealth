use crate::{fhir_client::ServerCTX, services::AppState};
use clap::ValueEnum;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType},
    types::FHIRString,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_repository::{
    Repository, admin::TenantAuthAdmin, types::tenant::CreateTenant, utilities::generate_id,
};
use std::sync::Arc;

#[derive(Debug, Clone, ValueEnum)]
pub enum SubscriptionTier {
    Free,
    Professional,
    Team,
    Unlimited,
}

impl From<SubscriptionTier> for String {
    fn from(tier: SubscriptionTier) -> Self {
        match tier {
            SubscriptionTier::Free => "free".to_string(),
            SubscriptionTier::Professional => "professional".to_string(),
            SubscriptionTier::Team => "team".to_string(),
            SubscriptionTier::Unlimited => "unlimited".to_string(),
        }
    }
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
    let services = services.transaction().await?;

    let new_tenant = TenantAuthAdmin::create(
        &*services.repo,
        &TenantId::System,
        CreateTenant {
            id: Some(TenantId::new(id.unwrap_or(generate_id(Some(16))))),
            subscription_tier: Some(subscription_tier.clone().into()),
        },
    )
    .await?;

    services
        .fhir_client
        .create(
            Arc::new(ServerCTX::system(new_tenant.id, ProjectId::System)),
            ResourceType::Project,
            Resource::Project(Project {
                id: Some(ProjectId::System.to_string()),
                name: Some(Box::new(FHIRString {
                    value: Some(ProjectId::System.to_string()),
                    ..Default::default()
                })),
                fhirVersion: Box::new(
                    oxidized_fhir_model::r4::generated::terminology::SupportedFhirVersion::R4(None),
                ),
                ..Default::default()
            }),
        )
        .await?;

    services.commit().await?;

    Ok(())
}
