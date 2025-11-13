use crate::{
    ServerEnvironmentVariables,
    auth_n::oidc::utilities::set_user_password,
    extract::path_tenant::{Project, ProjectIdentifier, TenantIdentifier},
    services::AppState,
    ui::{
        self,
        pages::{self, message::message_html},
    },
};
use axum::{
    Form,
    extract::{OriginalUri, Query, State},
    http::Uri,
};
use axum_extra::{extract::Cached, routing::TypedPath};
use maud::{Markup, html};
use oxidized_config::Config;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::{ProjectAuthAdmin, TenantAuthAdmin},
    types::{
        authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
        user::{AuthMethod, CreateUser, UserSearchClauses},
    },
};
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};

async fn send_email(
    config: &dyn Config<ServerEnvironmentVariables>,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), OperationOutcomeError> {
    let from_address = config.get(ServerEnvironmentVariables::EmailFromAddress)?;
    let api_key = config.get(ServerEnvironmentVariables::SendGridAPIKey)?;
    let sender = Sender::new(api_key, None);

    let m = Message::new(Email::new(&from_address))
        .set_subject(subject)
        .add_content(Content::new().set_content_type("text/html").set_value(body))
        .add_personalization(Personalization::new(Email::new(to)));

    let resp = sender.send(&m).await.map_err(|e| {
        tracing::error!("Failed to send email '{}'", e);
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Failed to send email".to_string(),
        )
    })?;

    tracing::info!("Email sent status: '{}'", resp.status());

    Ok(())
}

#[derive(TypedPath)]
#[typed_path("/password-reset")]
pub struct PasswordResetInitiate;

pub async fn password_reset_initiate_get(
    _: PasswordResetInitiate,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(Project(project)): Cached<Project>,
    uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    let response = pages::email_form::email_form_html(
        &tenant,
        &project,
        &pages::email_form::EmailInformation {
            continue_url: uri.path().to_string(),
        },
    );

    Ok(response)
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct PasswordResetFormInitiate {
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
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    uri: OriginalUri,
    form: axum::extract::Form<PasswordResetFormInitiate>,
) -> Result<Markup, OperationOutcomeError> {
    let user_search_results = TenantAuthAdmin::search(
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
        let password_reset_code = ProjectAuthAdmin::create(
            &*state.repo,
            &tenant,
            &project,
            CreateAuthorizationCode {
                membership: None,
                expires_in: Duration::from_secs(60 * 15), // 15 minutes
                kind: AuthorizationCodeKind::PasswordReset,
                user_id: user.id,
                client_id: None,
                pkce_code_challenge: None,
                pkce_code_challenge_method: None,
                redirect_uri: None,
                meta: None,
            },
        )
        .await?;

        let api_url_string = state.config.get(ServerEnvironmentVariables::APIURI)?;

        let api_url = Uri::try_from(&api_url_string).map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "API Url is invalid".to_string(),
            )
        })?;

        let redirection = Uri::builder()
            .authority(api_url.authority().unwrap().clone())
            .scheme(api_url.scheme().unwrap().clone())
            .path_and_query(
                uri.path().to_string().replace(
                    PasswordResetInitiate.to_uri().path(),
                    PasswordResetVerify.to_uri().path(),
                ) + "?code="
                    + &password_reset_code.code,
            )
            .build()
            .unwrap();

        let password_reset_html = ui::email::base::base(
            &api_url,
            html! {
                a href=(redirection.to_string()) style="color:#ffffff;font-size:14px;font-weight:bold;background-color:#6366f1;display:inline-block;padding:12px 20px;text-decoration:none" target="_blank" {
                    span { "Reset Password" }
                }
            },
        );

        let email = user.email.as_ref().ok_or_else(|| {
            OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "User does not have an email associated.".to_string(),
            )
        })?;

        send_email(
            &*state.config,
            email,
            "Password Reset",
            &password_reset_html.into_string(),
        )
        .await?;

        Ok(message_html(
            &tenant,
            &project_resource.0,
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
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Markup, OperationOutcomeError> {
    if let Some(code) = ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::read(
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
            &tenant,
            &project_resource,
            html! {
                div {}
                h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl "{
                    "Set your password"}
                form class="space-y-4 md:space-y-6" action=(uri.path().to_string()) method="POST"{
                    input type="hidden" id="code" name="code" value=(query.code) {}
                    label for="password" class="block mb-2 text-sm font-medium text-gray-900"{"Enter your Password"}
                    input type="password" id="password" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5" required="" name="password" {}
                    label for="password_confirm" class="block mb-2 text-sm font-medium text-gray-900"  {"Confirm your Password"}
                    input type="password" id="password_confirm" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5" required="" name="password_confirm" {}
                    button type="submit" class="w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center"{"Continue"}
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
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Form(body): Form<PasswordVerifyPOSTBODY>,
) -> Result<Markup, OperationOutcomeError> {
    if body.password != body.password_confirm {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Passwords do not match.".to_string(),
        ));
    }

    if let Some(code) = ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::read(
        &*state.repo,
        &tenant,
        &project,
        &body.code,
    )
    .await?
    {
        ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
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
            TenantAuthAdmin::<CreateUser, _, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
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

        Ok(message_html(
            &tenant,
            &project_resource,
            html! {"Password has been reset successfully."},
        ))
    } else {
        Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "Invalid Password reset code.".to_string(),
        ))?
    }
}
