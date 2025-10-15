use crate::{
    fhir_client::{FHIRServerClient, ServerCTX, ServerClientConfig},
    services::AppState,
};
use clap::ValueEnum;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{ProjectId, TenantId, tenant::CreateTenant},
    utilities::generate_id,
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
    let transaction_repo = Arc::new(services.repo.transaction().await?);
    {
        let fhir_client = FHIRServerClient::new(ServerClientConfig::new(
            transaction_repo.clone(),
            services.search.clone(),
            services.terminology.clone(),
        ));

        let new_tenant = TenantAuthAdmin::create(
            &*transaction_repo.clone(),
            &TenantId::System,
            CreateTenant {
                id: Some(TenantId::new(id.unwrap_or(generate_id(Some(16))))),
                subscription_tier: Some(subscription_tier.clone().into()),
            },
        )
        .await?;

        fhir_client
            .create(
                Arc::new(ServerCTX::root(
                    TenantId::new(new_tenant.id),
                    ProjectId::System,
                )),
                ResourceType::Project,
                Resource::Project(Project {
                    id: Some(ProjectId::System.to_string()),
                    name: Some(Box::new(ProjectId::System.to_string())),
                    fhirVersion: Box::new(
                        oxidized_fhir_model::r4::generated::terminology::SupportedFhirVersion::R4(
                            None,
                        ),
                    ),
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
