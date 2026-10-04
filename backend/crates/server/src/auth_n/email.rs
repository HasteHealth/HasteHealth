use crate::{
    config::{EmailConfig, ServerConfig},
    route_path::api_v1_oidc_path,
    services::ServerState,
    ui::pages::global_auth::INVITATION_ROUTE,
};
use axum::http::Uri;
use email_address::EmailAddress;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    Repository,
    admin::ProjectModelAdmin,
    types::{
        authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
        user::User,
    },
};
use maud::{Markup, html};
use sendgrid::v3::{Content, Email, Personalization, Sender};
use std::{str::FromStr, time::Duration};
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
    config: &Option<EmailConfig>,
    to: &EmailAddress,
    subject: &str,
    body: &str,
) -> Result<(), OperationOutcomeError> {
    let email_config = config.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            "Email configuration is not set".to_string(),
        )
    })?;

    match email_config {
        EmailConfig::SendGrid {
            from_address,
            api_key,
        } => {
            let sender = Sender::new(api_key, None);

            let m = sendgrid::v3::Message::new(Email::new(from_address))
                .set_subject(subject)
                .add_content(Content::new().set_content_type("text/html").set_value(body))
                .add_personalization(Personalization::new(Email::new(to.as_str())));

            let resp = sender.send(&m).await.map_err(|e| {
                tracing::error!("Failed to send email '{}'", e);
                tracing::error!("{}", report(&e));
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Failed to send email".to_string(),
                )
            })?;

            tracing::info!("Email sent status: '{}'", resp.status());

            Ok(())
        }
        EmailConfig::Log => {
            tracing::info!(to = %to, subject, body, "Email logged instead of sent");

            Ok(())
        }
    }
}

/// Emails a one-time code. The subject carries it too, so it can be read from
/// a notification.
pub async fn send_email_code(
    config: &Option<EmailConfig>,
    api_uri: &str,
    to: &EmailAddress,
    code: &str,
    valid_for: Duration,
) -> Result<(), OperationOutcomeError> {
    let api_url = Uri::try_from(api_uri).map_err(|_| {
        OperationOutcomeError::fatal(IssueType::exception(), "API Url is invalid".to_string())
    })?;

    let minutes = valid_for.as_secs() / 60;

    let body = crate::ui::email::base::base(
        &api_url,
        &html! {
            div style="padding-top: 24px;" {
                "Enter this code to continue to Haste Health:"
            }
            div style="font-size: 32px; font-weight: 700; letter-spacing: 8px; padding: 24px 0px;" {
                (code)
            }
            div style="color: #475569; font-size: 14px;" {
                "The code expires in " (minutes) " minutes and only works in the browser where you requested it."
            }
            div style="color: #475569; font-size: 14px; padding-top: 12px;" {
                "If you did not request this, you can ignore this email."
            }
        },
    );

    send_email(
        config,
        to,
        &format!("{code} is your Haste Health code"),
        &body.into_string(),
    )
    .await
}

pub fn invitation_subject(tenant_name: &str) -> String {
    format!("You're invited to {tenant_name} on Haste Health")
}

/// Emails an invited address a link to accept or decline.
pub async fn send_invitation_email(
    config: &ServerConfig,
    tenant: &TenantId,
    tenant_name: &str,
    to: &EmailAddress,
    code: &str,
    valid_for: Duration,
) -> Result<(), OperationOutcomeError> {
    let invalid_api_url =
        || OperationOutcomeError::fatal(IssueType::exception(), "API Url is invalid".to_string());

    let mut link = Url::parse(&config.api_uri).map_err(|_| invalid_api_url())?;
    link.set_path(INVITATION_ROUTE);
    link.query_pairs_mut()
        .append_pair("tenant", tenant.as_ref())
        .append_pair("code", code);

    let days = valid_for.as_secs() / (24 * 60 * 60);

    let body = crate::ui::email::base::base(
        &Uri::try_from(link.as_str()).map_err(|_| invalid_api_url())?,
        &html! {
            div style="padding-top: 24px;" {
                span style="font-weight: 600;" { (tenant_name) }
                " invited " span style="font-weight: 600;" { (to.as_str()) }
                " to join its workspace on Haste Health."
            }
            div style="padding-top: 12px;" {
                "Nothing happens until you accept. Review the invitation to accept it or decline it:"
            }
            div style="padding: 24px 0px;" {
                a href=(link.as_str()) style="color:#ffffff;font-size:14px;font-weight:bold;background-color:#00786f;display:inline-block;padding:12px 24px;text-decoration:none" target="_blank" {
                    span { "Review invitation" }
                }
            }
            div style="color: #475569; font-size: 14px;" {
                "The link works for " (days) " days. If you don't know this workspace, you can ignore this email."
            }
        },
    );

    send_email(
        &config.email,
        to,
        &invitation_subject(tenant_name),
        &body.into_string(),
    )
    .await
}

#[derive(Default)]
pub struct Message {
    pub subject: Option<String>,
    pub body: Option<Markup>,
}

pub async fn send_password_reset_email<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    user: &User,
    message: Message,
) -> Result<(), OperationOutcomeError> {
    let password_reset_code = ProjectModelAdmin::create(
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

    let api_url_string = &state.config.api_uri;

    let mut api_url = Url::parse(api_url_string).map_err(|_| {
        OperationOutcomeError::fatal(IssueType::exception(), "API Url is invalid".to_string())
    })?;

    api_url.set_path(
        api_v1_oidc_path(tenant, project)
            .join(format!(
                "interactions{}",
                crate::auth_n::oidc::routes::interactions::password_reset::PasswordResetVerify
            ))
            .to_str()
            .unwrap_or_default(),
    );

    api_url.set_query(Some(format!("code={}", password_reset_code.code).as_str()));

    let reset_button = crate::ui::email::base::base(
        &Uri::try_from(api_url.as_str()).map_err(|_| {
            OperationOutcomeError::fatal(IssueType::exception(), "API Url is invalid".to_string())
        })?,
        &html! {
            @if let Some(message) = message.body {
                div style="padding-top: 24px;" {
                    (message)
                }
            }
            div style="font-weight: 600; padding: 24px 0px;" { "To verify your email and set your password click below." }
            a href=(api_url.as_str()) style="color:#ffffff;font-size:14px;font-weight:bold;background-color:#00786f;display:inline-block;padding:12px 24px;text-decoration:none" target="_blank" {
                span { "Reset Password" }
            }
        },
    );

    let email_str = user.email.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            "User does not have an email associated.".to_string(),
        )
    })?;

    send_email(
        &state.config.email,
        &EmailAddress::from_str(email_str).map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::invalid(),
                "User has an invalid email associated.".to_string(),
            )
        })?,
        message.subject.as_deref().unwrap_or("Password Reset"),
        &reset_button.into_string(),
    )
    .await?;

    Ok(())
}
