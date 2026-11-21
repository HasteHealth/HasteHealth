//! Users a tenant creates for someone else. Each is an invitation until the
//! person proves they own the address and accepts. Acceptance sets
//! `emailVerified`, which login requires and clients cannot set.

use crate::{
    auth_n::email::send_invitation_email,
    config::ServerConfig,
    fhir_client::{FHIRServerClient, ServerCTX},
    services::ServerState,
};
use email_address::EmailAddress;
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, User as UserResource},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    Repository,
    admin::{ProjectModelAdmin, TenantModelAdmin},
    types::{
        authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
        tenant::CreateTenant,
        user::{CreateUser, UpdateUser, User},
    },
};
use std::{str::FromStr, sync::Arc, time::Duration};

/// How long an invitation link works.
pub const INVITATION_VALID_FOR: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// `emailVerified`, or false when absent.
pub fn resource_email_verified(user: &UserResource) -> bool {
    user.emailVerified
        .as_ref()
        .and_then(|verified| verified.value)
        .unwrap_or(false)
}

/// Sets `emailVerified`, keeping the element's other fields.
pub fn set_resource_email_verified(user: &mut UserResource, verified: bool) {
    user.emailVerified.get_or_insert_default().value = Some(verified);
}

/// The tenant's display name, or its id if it has none.
pub async fn tenant_display_name<Repo: Repository>(repo: &Repo, tenant: &TenantId) -> String {
    TenantModelAdmin::<CreateTenant, _, _, _, _>::read(
        repo,
        &TenantId::System,
        &tenant.as_ref().to_string(),
    )
    .await
    .ok()
    .flatten()
    .and_then(|tenant| tenant.display_name)
    .unwrap_or_else(|| tenant.as_ref().to_string())
}

async fn email_invitation<Repo: Repository>(
    repo: &Repo,
    config: &ServerConfig,
    tenant: &TenantId,
    user_id: &str,
    to: &EmailAddress,
) -> Result<(), OperationOutcomeError> {
    let code = ProjectModelAdmin::create(
        repo,
        tenant,
        &ProjectId::System,
        CreateAuthorizationCode {
            membership: None,
            expires_in: INVITATION_VALID_FOR,
            kind: AuthorizationCodeKind::Invitation,
            user_id: user_id.to_string(),
            client_id: None,
            pkce_code_challenge: None,
            pkce_code_challenge_method: None,
            redirect_uri: None,
            meta: None,
        },
    )
    .await?;

    let tenant_name = tenant_display_name(repo, tenant).await;

    send_invitation_email(
        config,
        tenant,
        &tenant_name,
        to,
        &code.code,
        INVITATION_VALID_FOR,
    )
    .await
}

/// Emails a new user a link to accept or decline. Failures are only logged:
/// the user can still accept by logging in.
pub async fn send_invitation<Repo: Repository>(
    repo: &Repo,
    config: &ServerConfig,
    tenant: &TenantId,
    user: &UserResource,
) {
    let Some(user_id) = user.id.as_deref() else {
        return;
    };
    let Some(email) = user.email.as_ref().and_then(|email| email.value.as_deref()) else {
        return;
    };
    let Ok(to) = EmailAddress::from_str(email) else {
        tracing::warn!(tenant = %tenant, user = user_id, "invited user has no valid email address");
        return;
    };

    if let Err(error) = email_invitation(repo, config, tenant, user_id, &to).await {
        tracing::warn!(tenant = %tenant, user = user_id, ?error, "invitation email not sent");
    }
}

/// The user an unexpired invitation `code` is for.
pub async fn find_invitation<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    code: &str,
) -> Result<Option<User>, OperationOutcomeError> {
    let Some(code) = ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::read(
        repo,
        tenant,
        &ProjectId::System,
        &code.to_string(),
    )
    .await?
    else {
        return Ok(None);
    };

    if code.kind != AuthorizationCodeKind::Invitation || code.is_expired.unwrap_or(true) {
        return Ok(None);
    }

    TenantModelAdmin::<CreateUser, _, _, _, _>::read(repo, tenant, &code.user_id).await
}

/// Spends an invitation code.
pub async fn delete_invitation_code<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    code: &str,
) -> Result<(), OperationOutcomeError> {
    ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
        repo,
        tenant,
        &ProjectId::System,
        &code.to_string(),
    )
    .await
}

fn system_ctx<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
) -> Arc<ServerCTX<FHIRServerClient<Repo, Search, Terminology>>> {
    Arc::new(ServerCTX::system(
        tenant.clone(),
        ProjectId::System,
        state.fhir_client.clone(),
        state.rate_limit.clone(),
    ))
}

/// Accepts the user: sets `emailVerified` on the `User` resource so the row
/// follows, or on the row alone if there is no resource.
pub async fn accept<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    user: User,
) -> Result<User, OperationOutcomeError> {
    if user.email_verified {
        return Ok(user);
    }

    let ctx = system_ctx(state, &user.tenant);

    match state
        .fhir_client
        .read(ctx.clone(), ResourceType::User, user.id.clone())
        .await?
    {
        Some(Resource::User(mut resource)) => {
            set_resource_email_verified(&mut resource, true);
            state
                .fhir_client
                .update(
                    ctx,
                    ResourceType::User,
                    user.id.clone(),
                    Resource::User(resource),
                )
                .await?;
        }
        _ => {
            TenantModelAdmin::<CreateUser, _, _, _, String>::update(
                state.repo.as_ref(),
                &user.tenant,
                UpdateUser {
                    id: user.id.clone(),
                    email: None,
                    role: None,
                    method: None,
                    provider_id: None,
                    password: None,
                    email_verified: Some(true),
                },
            )
            .await?;
        }
    }

    Ok(User {
        email_verified: true,
        ..user
    })
}

/// Deletes an unaccepted user, through its `User` resource if it has one.
/// Its memberships cascade with the row.
pub async fn decline<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    user: &User,
) -> Result<(), OperationOutcomeError> {
    if user.email_verified {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "That account was accepted already. Ask the workspace to remove it.".to_string(),
        ));
    }

    let ctx = system_ctx(state, &user.tenant);

    match state
        .fhir_client
        .read(ctx.clone(), ResourceType::User, user.id.clone())
        .await?
    {
        Some(Resource::User(_)) => {
            state
                .fhir_client
                .delete_instance(ctx, ResourceType::User, user.id.clone())
                .await?;
        }
        _ => {
            TenantModelAdmin::<CreateUser, _, _, _, _>::delete(
                state.repo.as_ref(),
                &user.tenant,
                &user.id,
            )
            .await?;
        }
    }

    Ok(())
}
