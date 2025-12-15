use crate::{
    auth_n::oidc::utilities::set_user_password, fhir_client::ServerCTX, services::AppState,
};
use clap::ValueEnum;
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType, User},
    terminology::UserRole,
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::tenant::{CreateTenant, Tenant},
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

pub async fn create_user<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: &AppState<Repo, Search, Terminology>,
    tenant: &TenantId,
    email: &str,
    password: &str,
    user_role: UserRole,
) -> Result<User, OperationOutcomeError> {
    let ctx = Arc::new(ServerCTX::system(
        tenant.clone(),
        ProjectId::System,
        services.fhir_client.clone(),
    ));

    let user = services
        .fhir_client
        .create(
            ctx,
            ResourceType::User,
            Resource::User(User {
                role: Box::new(user_role),
                email: Some(Box::new(FHIRString {
                    value: Some(email.to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }),
        )
        .await?;

    let user = match user {
        Resource::User(user) => user,
        _ => panic!("Created resource is not a User"),
    };

    let user_id = user.id.clone().unwrap();

    set_user_password(&*services.repo, &tenant, email, &user_id, password).await?;

    Ok(user)
}

struct CreateTenantOutput {
    pub tenant: Tenant,
    pub owner: User,
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
    owner_email: &str,
    owner_password: &str,
) -> Result<CreateTenantOutput, OperationOutcomeError> {
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
            Arc::new(ServerCTX::system(
                new_tenant.id.clone(),
                ProjectId::System,
                services.fhir_client.clone(),
            )),
            ResourceType::Project,
            Resource::Project(Project {
                id: Some(ProjectId::System.to_string()),
                name: Box::new(FHIRString {
                    value: Some(ProjectId::System.to_string()),
                    ..Default::default()
                }),
                fhirVersion: Box::new(
                    haste_fhir_model::r4::generated::terminology::SupportedFhirVersion::R4(None),
                ),
                ..Default::default()
            }),
        )
        .await?;

    let user = create_user(
        &services,
        &new_tenant.id,
        &owner_email,
        &owner_password,
        UserRole::Owner(None),
    )
    .await?;

    services.commit().await?;

    Ok(CreateTenantOutput {
        tenant: new_tenant,
        owner: user,
    })
}
