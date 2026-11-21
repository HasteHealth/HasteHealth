use crate::auth_n::constant_time;
use crate::auth_n::error::AuthError;
use crate::{
    auth_n::{
        email::{Message, invitation_subject, send_password_reset_email},
        invitations,
        oidc::{hardcoded_clients::admin_app, utilities::set_user_password},
    },
    extract::{
        csrf_token::CSRFToken,
        path_tenant::{Project, ProjectIdentifier},
    },
    services::ServerState,
    ui::{
        components::TenantContext,
        pages::{self, message::message_html},
        styles::{BUTTON, HEADING, INPUT, LABEL, LINK},
    },
};
use axum::{
    Form,
    extract::{OriginalUri, Query, State},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
    Repository,
    admin::{ProjectModelAdmin, TenantModelAdmin},
    types::{
        authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
        user::{AuthMethod, CreateUser, UserSearchClauses},
    },
};
use maud::{Markup, html};
use serde::Deserialize;
use std::sync::Arc;

use crate::auth_n::oidc::utilities::revoke_refresh_tokens;

#[derive(TypedPath)]
#[typed_path("/password-reset")]
pub struct PasswordResetInitiate;

pub async fn password_reset_initiate_get(
    _: PasswordResetInitiate,
    Cached(TenantContext { tenant, branding }): Cached<TenantContext>,
    Cached(Project(project)): Cached<Project>,
    CSRFToken(csrf_token): CSRFToken,
    uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    let response = pages::email_form::email_form_html(
        &tenant,
        Some(&project),
        &csrf_token,
        &pages::email_form::EmailInformation {
            continue_url: uri.path().to_string(),
        },
        Some(&branding),
    );

    Ok(response)
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct PasswordResetFormInitiate {
    pub csrf_token: String,
    pub email: String,
}

pub async fn password_reset_initiate_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: PasswordResetInitiate,
    Cached(TenantContext { tenant, branding }): Cached<TenantContext>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    project_resource: Project,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    CSRFToken(csrf_token): CSRFToken,
    form: axum::extract::Form<PasswordResetFormInitiate>,
) -> Result<Markup, OperationOutcomeError> {
    if !constant_time::str_eq(&form.csrf_token, &csrf_token) {
        return Err(AuthError::InvalidCsrfToken.into());
    }

    let user_search_results = TenantModelAdmin::search(
        &*state.repo,
        &tenant,
        &UserSearchClauses {
            email: Some(form.email.clone()),
            role: None,
            method: Some(AuthMethod::EmailPassword),
        },
    )
    .await?;

    // The answer is the same whether or not the address belongs to a user, so
    // this form cannot be used to find out which addresses have accounts. The
    // email is sent off the request for the same reason: waiting for it would
    // make a known address answer measurably slower than an unknown one.
    if let Some(user) = user_search_results.into_iter().next() {
        // Tell an invited user that setting a password accepts the invitation.
        let message = if user.email_verified {
            Message::default()
        } else {
            let tenant_name = branding.0.as_deref().unwrap_or(tenant.as_ref());
            Message {
                subject: Some(invitation_subject(tenant_name)),
                body: Some(html! {
                    span class="font-semibold" { (tenant_name) }
                    " created an account for this address. Setting a password accepts the invitation. If you don't know this workspace, ignore this email and nothing will happen."
                }),
            }
        };

        let state = state.clone();
        let tenant = tenant.clone();
        let project = project.clone();

        tokio::spawn(async move {
            if let Err(error) =
                send_password_reset_email(state.as_ref(), &tenant, &project, &user, message).await
            {
                tracing::error!(?error, "Failed to send password reset email");
            }
        });
    } else {
        tracing::info!("Password reset requested for an unknown email address");
    }

    Ok(message_html(
        Some(&tenant),
        Some(&project_resource.0),
        &html! {"If an account exists for that email address, a message will arrive in the next few minutes with the next steps to reset your password."},
        Some(&branding),
    ))
}

#[derive(TypedPath)]
#[typed_path("/password-reset-verify")]
pub struct PasswordResetVerify;

#[derive(Deserialize)]
pub struct PasswordResetVerifyQuery {
    code: String,
}

pub async fn password_reset_verify_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: PasswordResetVerify,
    uri: OriginalUri,
    query: Query<PasswordResetVerifyQuery>,
    Cached(TenantContext { tenant, branding }): Cached<TenantContext>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Markup, OperationOutcomeError> {
    if let Some(code) = ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::read(
        &*state.repo,
        &tenant,
        &project,
        &query.code,
    )
    .await?
    {
        if code.kind != AuthorizationCodeKind::PasswordReset {
            return Err(AuthError::InvalidPasswordResetCode.into());
        }
        if code.is_expired.unwrap_or(true) {
            return Err(AuthError::ExpiredPasswordResetCode.into());
        }
        Ok(message_html(
            Some(&tenant),
            Some(&project_resource),
            &html! {
                h1 class=(HEADING) { "Set your password" }
                form class="space-y-4 md:space-y-6" action=(uri.path().to_string()) method="POST" {
                    input type="hidden" id="code" name="code" value=(query.code) {}
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    div {
                        label for="password" class=(LABEL) { "Enter your Password" }
                        input type="password" id="password" placeholder="••••••••" class=(INPUT) required="" name="password" {}
                    }
                    div {
                        label for="password_confirm" class=(LABEL) { "Confirm your Password" }
                        input type="password" id="password_confirm" placeholder="••••••••" class=(INPUT) required="" name="password_confirm" {}
                    }
                    button type="submit" class=(BUTTON) { "Continue" }
                }
            },
            Some(&branding),
        ))
    } else {
        Err(AuthError::InvalidPasswordResetCode)?
    }
}

#[derive(Deserialize)]
pub struct PasswordVerifyPOSTBODY {
    csrf_token: String,
    code: String,
    password: String,
    password_confirm: String,
}

pub async fn password_reset_verify_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: PasswordResetVerify,
    Cached(TenantContext { tenant, branding }): Cached<TenantContext>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    CSRFToken(csrf_token): CSRFToken,
    Form(body): Form<PasswordVerifyPOSTBODY>,
) -> Result<Markup, OperationOutcomeError> {
    if !constant_time::str_eq(&body.csrf_token, &csrf_token) {
        return Err(AuthError::InvalidCsrfToken.into());
    }

    if body.password != body.password_confirm {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Passwords do not match.".to_string(),
        ));
    }

    if let Some(code) = ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::read(
        &*state.repo,
        &tenant,
        &project,
        &body.code,
    )
    .await?
    {
        if code.kind != AuthorizationCodeKind::PasswordReset {
            return Err(AuthError::InvalidPasswordResetCode.into());
        }
        ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
            &*state.repo,
            &tenant,
            &project,
            &body.code,
        )
        .await?;
        if code.is_expired.unwrap_or(true) {
            return Err(AuthError::ExpiredPasswordResetCode.into());
        }

        let Some(user) =
            TenantModelAdmin::<CreateUser, _, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
                .await?
        else {
            return Err(OperationOutcomeError::error(
                IssueType::not_found(),
                "User not found.".to_string(),
            ));
        };

        let email = user.email.as_ref().ok_or(AuthError::UserWithoutEmail)?;

        set_user_password(&*state.repo, &tenant, email, &user.id, &body.password).await?;
        // Whoever held a refresh token before the reset must not keep access.
        revoke_refresh_tokens(&*state.repo, &tenant, &user.id).await?;

        // The emailed link proves the address, so this accepts an invitation.
        invitations::accept(state.as_ref(), user).await?;

        let admin_app_url = admin_app::redirect_url(state.config.as_ref(), &tenant, &project);

        Ok(message_html(
            Some(&tenant),
            Some(&project_resource),
            &html! { span {
                    "Password has been reset successfully. "
                    @if let Some(admin_app_url) = admin_app_url {
                        "Go to the Admin App "
                        a class=(LINK) href=(admin_app_url) { "here" }
                        "."
                    }
                }
            },
            Some(&branding),
        ))
    } else {
        Err(AuthError::InvalidPasswordResetCode)?
    }
}
