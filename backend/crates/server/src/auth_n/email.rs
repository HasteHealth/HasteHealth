use std::time::Duration;

use crate::{ServerEnvironmentVariables, route_path::api_v1_oidc_path, services::AppState};
use axum::http::Uri;
use haste_config::Config;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::{
        authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
        user::User,
    },
};
use maud::{Markup, html};
use sendgrid::v3::{Content, Email, Personalization, Sender};
use url::Url;

fn report(mut err: &dyn std::error::Error) -> String {
    let mut s = format!("{}", err);
    while let Some(src) = err.source() {
        s = format!("{}\n\nCaused by: {}", s, src);
        err = src;
    }
    s
}

pub async fn send_email(
    config: &dyn Config<ServerEnvironmentVariables>,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), OperationOutcomeError> {
    let from_address = config.get(ServerEnvironmentVariables::EmailFromAddress)?;
    let api_key = config.get(ServerEnvironmentVariables::SendGridAPIKey)?;
    let sender = Sender::new(&api_key, None);

    let m = sendgrid::v3::Message::new(Email::new(&from_address))
        .set_subject(subject)
        .add_content(Content::new().set_content_type("text/html").set_value(body))
        .add_personalization(Personalization::new(Email::new(to)));

    let resp = sender.send(&m).await.map_err(|e| {
        tracing::error!("Failed to send email '{}'", e);
        tracing::error!("{}", report(&e));
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Failed to send email".to_string(),
        )
    })?;

    tracing::info!("Email sent status: '{}'", resp.status());

    Ok(())
}

pub struct Message {
    pub subject: Option<String>,
    pub body: Option<Markup>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            subject: None,
            body: None,
        }
    }
}

pub async fn send_password_reset_email<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &AppState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    user: &User,
    message: Message,
) -> Result<(), OperationOutcomeError> {
    let password_reset_code = ProjectAuthAdmin::create(
        &*state.repo,
        tenant,
        project,
        CreateAuthorizationCode {
            membership: None,
            expires_in: Duration::from_secs(60 * 30), // 30 minutes
            kind: AuthorizationCodeKind::PasswordReset,
            user_id: user.id.to_string(),
            client_id: None,
            pkce_code_challenge: None,
            pkce_code_challenge_method: None,
            redirect_uri: None,
            meta: None,
        },
    )
    .await?;

    let api_url_string = state.config.get(ServerEnvironmentVariables::APIURI)?;

    let mut api_url = Url::parse(&api_url_string).map_err(|_| {
        OperationOutcomeError::fatal(IssueType::Exception(None), "API Url is invalid".to_string())
    })?;

    api_url.set_path(
        api_v1_oidc_path(tenant, project)
            .join(&format!(
                "interactions{}",
                crate::auth_n::oidc::routes::interactions::password_reset::PasswordResetVerify
                    .to_string()
            ))
            .to_str()
            .unwrap_or_default(),
    );

    api_url.set_query(Some(format!("code={}", password_reset_code.code).as_str()));

    let reset_button = crate::ui::email::base::base(
        &Uri::try_from(api_url.as_str()).map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "API Url is invalid".to_string(),
            )
        })?,
        html! {
            @if let Some(message) = message.body {
                div style="padding-top: 24px;" {
                    (message)
                }
            }
            div style="font-weight: 600; padding: 24px 0px;" { "To verify your email and set your password click below." }
            a href=(api_url.as_str()) style="color:#ffffff;font-size:14px;font-weight:bold;background-color:#ff6900;display:inline-block;padding:12px 24px;text-decoration:none" target="_blank" {
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
        message.subject.as_deref().unwrap_or("Password Reset"),
        &reset_button.into_string(),
    )
    .await?;

    Ok(())
}
