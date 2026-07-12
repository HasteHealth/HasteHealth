use crate::{
    auth_n::{
        email::{Message, send_password_reset_email},
        oidc::{hardcoded_clients::admin_app, utilities::set_user_password},
    },
    extract::{
        csrf_token::CSRFToken,
        path_tenant::{Project, ProjectIdentifier, TenantIdentifier},
    },
    services::ServerState,
    ui::pages::{self, message::message_html},
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
        authorization_code::CreateAuthorizationCode,
        user::{AuthMethod, CreateUser, UserSearchClauses},
    },
};
use maud::{Markup, html};
use serde::Deserialize;
use std::sync::Arc;

#[derive(TypedPath)]
#[typed_path("/password-reset")]
pub struct PasswordResetInitiate;

pub async fn password_reset_initiate_get(
    _: PasswordResetInitiate,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
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
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: PasswordResetInitiate,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    project_resource: Project,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    CSRFToken(csrf_token): CSRFToken,
    form: axum::extract::Form<PasswordResetFormInitiate>,
) -> Result<Markup, OperationOutcomeError> {
    if form.csrf_token != csrf_token {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid CSRF Token".to_string(),
        ));
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

    if let Some(user) = user_search_results.into_iter().next() {
        send_password_reset_email(state.as_ref(), &tenant, &project, &user, Message::default())
            .await?;

        Ok(message_html(
            Some(&tenant),
            Some(&project_resource.0),
            html! {"An email will arrive in the next few minutes with the next steps to reset your password."},
        ))
    } else {
        Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "No user found with provided email address.".to_string(),
        ))?
    }
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
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
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
        if code.is_expired.unwrap_or(true) {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Password reset code has expired.".to_string(),
            ));
        }
        Ok(message_html(
            Some(&tenant),
            Some(&project_resource),
            html! {
                div {}
                h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl "{
                    "Set your password"}
                form class="space-y-4 md:space-y-6" action=(uri.path().to_string()) method="POST"{
                    input type="hidden" id="code" name="code" value=(query.code) {}
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    label for="password" class="block mb-2 text-sm font-medium text-gray-900"{"Enter your Password"}
                    input type="password" id="password" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-brand-600 focus:border-brand-600 block w-full p-2.5" required="" name="password" {}
                    label for="password_confirm" class="block mb-2 text-sm font-medium text-gray-900"  {"Confirm your Password"}
                    input type="password" id="password_confirm" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-brand-600 focus:border-brand-600 block w-full p-2.5" required="" name="password_confirm" {}
                    button type="submit" class="cursor-pointer w-full text-white bg-brand-600 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center"{"Continue"}
                }
            },
        ))
    } else {
        Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "Invalid Password reset code.".to_string(),
        ))?
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
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    CSRFToken(csrf_token): CSRFToken,
    Form(body): Form<PasswordVerifyPOSTBODY>,
) -> Result<Markup, OperationOutcomeError> {
    if body.csrf_token != csrf_token {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid CSRF Token".to_string(),
        ));
    }

    if body.password != body.password_confirm {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
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
        ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
            &*state.repo,
            &tenant,
            &project,
            &body.code,
        )
        .await?;
        if code.is_expired.unwrap_or(true) {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Password reset code has expired.".to_string(),
            ));
        }

        let Some(user) =
            TenantModelAdmin::<CreateUser, _, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
                .await?
        else {
            return Err(OperationOutcomeError::error(
                IssueType::NotFound(None),
                "User not found.".to_string(),
            ));
        };

        let email = user.email.as_ref().ok_or_else(|| {
            OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "User does not have an email associated.".to_string(),
            )
        })?;

        set_user_password(&*state.repo, &tenant, &email, &user.id, &body.password).await?;

        let admin_app_url = admin_app::redirect_url(state.config.as_ref(), &tenant, &project);

        Ok(message_html(
            Some(&tenant),
            Some(&project_resource),
            html! { span {
                    "Password has been reset successfully. "
                    @if let Some(admin_app_url) = admin_app_url {
                        "Go to the Admin App "
                        a class="hover:underline cursor-pointer text-brand-600" href=(admin_app_url) { "here" }
                        "."
                    }
                }
            },
        ))
    } else {
        Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "Invalid Password reset code.".to_string(),
        ))?
    }
}
