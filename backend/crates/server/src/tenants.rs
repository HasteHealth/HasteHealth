use crate::{
    auth_n::oidc::utilities::set_user_password, fhir_client::ServerCTX, services::AppState,
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType, User},
    terminology::IssueType,
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, claims::SubscriptionTier};
use haste_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{
        tenant::{CreateTenant, Tenant},
        user::CreateUser,
    },
    utilities::generate_id,
};
use std::sync::Arc;

pub async fn create_user<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: &AppState<Repo, Search, Terminology>,
    tenant: &TenantId,
    user_resource: User,
    password: Option<&str>,
) -> Result<User, OperationOutcomeError> {
    let ctx = Arc::new(ServerCTX::system(
        tenant.clone(),
        ProjectId::System,
        services.fhir_client.clone(),
    ));

    let user = services
        .fhir_client
        .create(ctx, ResourceType::User, Resource::User(user_resource))
        .await?;

    let user = match user {
        Resource::User(user) => user,
        _ => panic!("Created resource is not a User"),
    };

    let user_id = user.id.clone().unwrap();

    if let Some(password) = password {
        set_user_password(
            &*services.repo,
            &tenant,
            &user
                .email
                .as_ref()
                .and_then(|e| e.value.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            &user_id,
            password,
        )
        .await?;
    }

    Ok(user)
}

pub struct CreateTenantOutput {
    pub tenant: Tenant,
    pub owner: haste_repository::types::user::User,
}

pub async fn create_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: &AppState<Repo, Search, Terminology>,
    tenant_id: Option<String>,
    _name: &str,
    subscription_tier: &SubscriptionTier,
    owner: haste_fhir_model::r4::generated::resources::User,
    owner_password: Option<&str>,
) -> Result<CreateTenantOutput, OperationOutcomeError> {
    let services = services.transaction().await?;

    let new_tenant = TenantAuthAdmin::create(
        &*services.repo,
        &TenantId::System,
        CreateTenant {
            id: Some(TenantId::new(tenant_id.unwrap_or(generate_id(Some(16))))),
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

    let user = create_user(&services, &new_tenant.id, owner, owner_password).await?;

    let Some(user_id) = user.id else {
        return Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            "The user ID is required to complete the tenant creation process.".to_string(),
        ));
    };

    let Some(user) = TenantAuthAdmin::<CreateUser, _, _, _, _>::read(
        services.repo.as_ref(),
        &new_tenant.id,
        &user_id,
    )
    .await?
    else {
        return Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            "The user does not exist after creation.".to_string(),
        ));
    };

    services.commit().await?;

    Ok(CreateTenantOutput {
        tenant: new_tenant,
        owner: user,
    })
}
