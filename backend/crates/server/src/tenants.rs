use crate::{
    auth_n::oidc::utilities::set_user_password, fhir_client::ServerCTX, services::ServerState,
    ui::components::TenantName,
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType, User},
    terminology::{IssueType, SupportedFhirVersion},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, claims::SubscriptionTier};
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::{
        tenant::{CreateTenant, Tenant},
        user::CreateUser,
    },
    utilities::{HOSTNAME_ID_MIN_LEN, generate_hostname_id, validate_hostname_id},
};
use std::sync::Arc;

pub async fn create_user<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    user_resource: User,
    password: Option<&str>,
) -> Result<User, OperationOutcomeError> {
    let ctx = Arc::new(ServerCTX::system(
        tenant.clone(),
        ProjectId::System,
        services.fhir_client.clone(),
        services.rate_limit.clone(),
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
            tenant,
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

pub async fn read_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: &ServerState<Repo, Search, Terminology>,
    tenant_id: &TenantId,
) -> Result<Tenant, OperationOutcomeError> {
    TenantModelAdmin::<CreateTenant, _, _, _, _>::read(
        services.repo.as_ref(),
        &TenantId::System,
        &tenant_id.as_ref().to_string(),
    )
    .await?
    .ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::not_found(),
            format!("Tenant '{}' was not found.", tenant_id),
        )
    })
}

pub fn tenant_name(tenant: &Tenant) -> TenantName {
    TenantName(tenant.display_name.clone())
}

pub const TENANT_ID_MIN_LEN: usize = HOSTNAME_ID_MIN_LEN;
pub const TENANT_ID_MAX_LEN: usize = 32;

/// Names that would read as something else in a URL path or an admin app
/// hostname. The system tenant is refused separately, by type.
const RESERVED_TENANT_IDS: &[&str] = &[
    "admin",
    "api",
    "auth",
    "www",
    "haste",
    "haste-health",
    "hastehealth",
    "login",
    "signup",
    "support",
    "help",
];

/// Domains that say nothing about an organisation.
const PUBLIC_MAIL_DOMAINS: &[&str] = &[
    "gmail.com",
    "googlemail.com",
    "outlook.com",
    "hotmail.com",
    "live.com",
    "msn.com",
    "yahoo.com",
    "ymail.com",
    "icloud.com",
    "me.com",
    "mac.com",
    "aol.com",
    "proton.me",
    "protonmail.com",
    "pm.me",
    "mail.com",
    "gmx.com",
    "gmx.net",
    "fastmail.com",
    "hey.com",
    "yandex.com",
    "zoho.com",
];

/// Checks an id chosen for a new tenant. The length cap leaves a project id
/// room in the hostname label the two share. Refuses the system tenant and
/// the reserved names.
pub fn validate_tenant_id(id: &str) -> Result<TenantId, OperationOutcomeError> {
    let invalid = |message: String| OperationOutcomeError::error(IssueType::invalid(), message);

    let length = id.chars().count();
    if !(TENANT_ID_MIN_LEN..=TENANT_ID_MAX_LEN).contains(&length) {
        return Err(invalid(format!(
            "A tenant id is {TENANT_ID_MIN_LEN} to {TENANT_ID_MAX_LEN} characters long."
        )));
    }

    validate_hostname_id(id)?;

    let tenant = TenantId::new(id.to_string());
    if tenant == TenantId::System || RESERVED_TENANT_IDS.contains(&id) {
        return Err(invalid(format!(
            "'{id}' is reserved. Choose another tenant id."
        )));
    }

    Ok(tenant)
}

fn random_tenant_id() -> TenantId {
    TenantId::new(generate_hostname_id(Some(16)))
}

/// A valid tenant id to propose for an email address: the organisation from a
/// work address, otherwise the mailbox name. It may already be taken.
pub fn suggested_tenant_id(email: &str) -> String {
    let (mailbox, domain) = email.rsplit_once('@').unwrap_or((email, ""));
    let domain = domain.to_ascii_lowercase();
    let base = if domain.is_empty() || PUBLIC_MAIL_DOMAINS.contains(&domain.as_str()) {
        mailbox
    } else {
        domain.split('.').next().unwrap_or(mailbox)
    };

    // Runs of anything but ASCII letters and digits become one hyphen.
    let id = base
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let id = id[..id.len().min(TENANT_ID_MAX_LEN)].trim_end_matches('-');

    validate_tenant_id(id)
        .unwrap_or_else(|_| random_tenant_id())
        .as_ref()
        .to_string()
}

/// `base`, a valid tenant id, with a short random suffix, for when `base` is
/// taken.
pub fn suggested_tenant_id_variant(base: &str) -> String {
    const SUFFIX_LEN: usize = 4;

    let base: String = base
        .chars()
        .take(TENANT_ID_MAX_LEN - SUFFIX_LEN - 1)
        .collect();
    let base = base.trim_end_matches('-');

    format!("{base}-{}", generate_hostname_id(Some(SUFFIX_LEN)))
}

pub async fn create_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    services: &ServerState<Repo, Search, Terminology>,
    tenant_id: Option<String>,
    subscription_tier: &SubscriptionTier,
    owner: haste_fhir_model::r4::generated::resources::User,
    owner_password: Option<&str>,
) -> Result<CreateTenantOutput, OperationOutcomeError> {
    let id = match tenant_id {
        Some(id) => validate_tenant_id(&id)?,
        None => random_tenant_id(),
    };

    let services = services.transaction().await?;

    let new_tenant = TenantModelAdmin::create(
        &*services.repo,
        &TenantId::System,
        CreateTenant {
            id: Some(id),
            subscription_tier: Some(subscription_tier.clone().into()),
            display_name: None,
            logo_data: None,
            logo_content_type: None,
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
                services.rate_limit.clone(),
            )),
            ResourceType::Project,
            Resource::Project(Project {
                id: Some(ProjectId::System.to_string()),
                name: Box::new(FHIRString {
                    value: Some(ProjectId::System.to_string()),
                    ..Default::default()
                }),
                fhirVersion: SupportedFhirVersion::r4(),
                ..Default::default()
            }),
        )
        .await?;

    let user = create_user(&services, &new_tenant.id, owner, owner_password).await?;

    let Some(user_id) = user.id else {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "The user ID is required to complete the tenant creation process.".to_string(),
        ));
    };

    let Some(user) = TenantModelAdmin::<CreateUser, _, _, _, _>::read(
        services.repo.as_ref(),
        &new_tenant.id,
        &user_id,
    )
    .await?
    else {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "The user does not exist after creation.".to_string(),
        ));
    };

    services.commit().await?;

    Ok(CreateTenantOutput {
        tenant: new_tenant,
        owner: user,
    })
}
