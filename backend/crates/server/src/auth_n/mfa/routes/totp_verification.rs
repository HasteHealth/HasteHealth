use crate::{
    auth_n::{
        mfa::utilities::user_mfa_to_totp,
        oidc::routes::route_string::tenant_route_string,
        session::{self, user::SessionAuthorizationState},
    },
    extract::{csrf_token::CSRFToken, path_tenant::TenantIdentifier},
    services::ServerState,
    ui::pages::mfa,
};
use axum::{
    Form,
    extract::{OriginalUri, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::{
        mfa::{UserMFACredential, UserMFACredentialCreate, UserMFASearchClaims},
        scope::UserId,
        user::User,
    },
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use url::form_urlencoded;

fn is_safe_local_redirect_path(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//")
}

pub fn totp_verification_route(tenant: &TenantId, redirect_to: &str) -> String {
    let route = tenant_route_string(tenant)
        .join("mfa")
        .join("totp-verification");

    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("redirect_to", redirect_to)
        .finish();

    format!("{}?{}", route.to_string_lossy(), query)
}

async fn get_required_totp_credentials<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    current_session: &Session,
) -> Result<(User, Vec<UserMFACredential>), OperationOutcomeError> {
    let user = match session::user::get_authorization_state(current_session).await? {
        Some(SessionAuthorizationState::MFARequired { user }) => user,
        _ => {
            return Err(OperationOutcomeError::error(
                IssueType::security(),
                "MFA verification is not required.".to_string(),
            ));
        }
    };

    let credentials = TenantModelAdmin::<UserMFACredentialCreate, _, _, _, _>::search(
        state.repo.as_ref(),
        tenant,
        &UserMFASearchClaims {
            tenant: tenant.clone(),
            user_id: UserId::new(user.id.clone()),
            is_active: Some(true),
        },
    )
    .await?
    .into_iter()
    .filter(|credential| credential.credential_type == "totp")
    .collect::<Vec<_>>();

    if credentials.is_empty() {
        return Err(OperationOutcomeError::error(
            IssueType::not_found(),
            "No active TOTP credential found for this user.".to_string(),
        ));
    }

    Ok((user, credentials))
}

#[derive(Deserialize)]
pub struct TOTPVerificationPOSTBody {
    pub csrf_token: String,
    pub otp_code: String,
}

#[derive(Deserialize)]
pub struct TOTPVerificationQuery {
    pub redirect_to: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/totp-verification")]
pub struct TOTPVerificationGET;

#[derive(TypedPath, Deserialize)]
#[typed_path("/totp-verification")]
pub struct TOTPVerificationPOST;

pub async fn totp_verification_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TOTPVerificationGET,
    Query(query): Query<TOTPVerificationQuery>,
    uri: OriginalUri,
    CSRFToken(csrf_token): CSRFToken,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    if !is_safe_local_redirect_path(&query.redirect_to) {
        return Err(OperationOutcomeError::error(
            IssueType::security(),
            "Invalid MFA redirect target.".to_string(),
        ));
    }

    let (_, credentials) =
        get_required_totp_credentials(state.as_ref(), &tenant, &current_session).await?;

    Ok(mfa::totp_verification::totp_entry_html(
        &tenant,
        &csrf_token,
        credentials[0].totp_digits as usize,
        &uri.to_string(),
        None,
    )
    .into_response())
}

pub async fn totp_verification_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TOTPVerificationPOST,
    Query(query): Query<TOTPVerificationQuery>,
    uri: OriginalUri,
    CSRFToken(csrf_token): CSRFToken,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    Form(form_data): Form<TOTPVerificationPOSTBody>,
) -> Result<Response, OperationOutcomeError> {
    if form_data.csrf_token != csrf_token {
        return Err(OperationOutcomeError::error(
            IssueType::security(),
            "Invalid CSRF token.".to_string(),
        ));
    }

    if !is_safe_local_redirect_path(&query.redirect_to) {
        return Err(OperationOutcomeError::error(
            IssueType::security(),
            "Invalid MFA redirect target.".to_string(),
        ));
    }

    let (user, credentials) =
        get_required_totp_credentials(state.as_ref(), &tenant, &current_session).await?;
    let digits = credentials[0].totp_digits as usize;

    let mut is_otp_valid = false;

    for credential in credentials {
        let totp = user_mfa_to_totp(
            state.secret_provider.as_ref(),
            &state.config,
            &user,
            credential,
        )
        .await?;

        if totp.check_current(&form_data.otp_code).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::security(),
                "Invalid verification code.".to_string(),
            )
        })? {
            is_otp_valid = true;
            break;
        }
    }

    if !is_otp_valid {
        return Ok(mfa::totp_verification::totp_entry_html(
            &tenant,
            &csrf_token,
            digits,
            &uri.to_string(),
            Some(vec![
                "Invalid verification code. Please try again.".to_string(),
            ]),
        )
        .into_response());
    }

    session::user::set_completed_authorization_state(&current_session, user).await?;

    Ok(Redirect::to(&query.redirect_to).into_response())
}
