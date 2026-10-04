//! `/auth/invitation`: the emailed invitation link. The link proves the
//! address, so the page only asks to accept or decline. Either answer spends
//! the link.

use crate::{
    auth_n::{
        global::routes::flow::{self, check_csrf},
        invitations,
    },
    extract::csrf_token::CSRFToken,
    services::ServerState,
    ui::pages::global_auth::{
        invitation_accepted_already_html, invitation_declined_html, invitation_html,
        invitation_invalid_html,
    },
};
use axum::{
    Form,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::Repository;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath, Deserialize)]
#[typed_path("/invitation")]
pub struct Invitation;

#[derive(Deserialize)]
pub struct InvitationQuery {
    pub tenant: String,
    pub code: String,
}

pub async fn invitation_get<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: Invitation,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Query(query): Query<InvitationQuery>,
) -> Result<Response, OperationOutcomeError> {
    let tenant = TenantId::new(query.tenant);

    let Some(user) =
        invitations::find_invitation(state.repo.as_ref(), &tenant, &query.code).await?
    else {
        return Ok(invitation_invalid_html().into_response());
    };

    if user.email_verified {
        return Ok(invitation_accepted_already_html().into_response());
    }

    let tenant_name = invitations::tenant_display_name(state.repo.as_ref(), &tenant).await;

    Ok(invitation_html(
        &csrf_token,
        tenant.as_ref(),
        &tenant_name,
        user.email.as_deref().unwrap_or_default(),
        &query.code,
    )
    .into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Accept,
    Decline,
}

#[derive(Deserialize)]
pub struct InvitationForm {
    pub csrf_token: String,
    pub tenant: String,
    pub code: String,
    pub decision: Decision,
}

pub async fn invitation_post<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: Invitation,
    CSRFToken(csrf_token): CSRFToken,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(session): Cached<Session>,
    Form(form): Form<InvitationForm>,
) -> Result<Response, OperationOutcomeError> {
    check_csrf(&csrf_token, &form.csrf_token)?;

    let tenant = TenantId::new(form.tenant);

    let Some(user) = invitations::find_invitation(state.repo.as_ref(), &tenant, &form.code).await?
    else {
        return Ok(invitation_invalid_html().into_response());
    };

    invitations::delete_invitation_code(state.repo.as_ref(), &tenant, &form.code).await?;

    if user.email_verified {
        return Ok(invitation_accepted_already_html().into_response());
    }

    match form.decision {
        Decision::Accept => {
            let user = invitations::accept(state.as_ref(), user).await?;

            flow::login_user(state.as_ref(), &session, user, None).await
        }
        Decision::Decline => {
            invitations::decline(state.as_ref(), &user).await?;

            let tenant_name = invitations::tenant_display_name(state.repo.as_ref(), &tenant).await;

            Ok(invitation_declined_html(&tenant_name).into_response())
        }
    }
}
