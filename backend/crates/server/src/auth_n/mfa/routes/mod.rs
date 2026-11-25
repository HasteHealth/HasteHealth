use crate::auth_n::error::AuthError;
use crate::auth_n::session::{self, user::SessionAuthorizationState};
use crate::extract::path_tenant::TenantIdentifier;
use crate::services::ServerState;
use axum::Router;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::Cached;
use axum_extra::routing::RouterExt;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::Repository;
use std::sync::Arc;
use tower_sessions::Session;

mod activate;
mod admin;
mod create;
mod delete;
pub mod totp_verification;

/// Whether a session in `state` is allowed to act on `url_tenant`'s MFA.
///
/// One session cookie is shared across every tenant and users are keyed by
/// `(tenant, id)`, so the session user's own tenant must match the one named in
/// the URL. The check holds whatever stage of login the session is at: the
/// fully logged-in `Complete` state the management routes need, and the
/// `MFARequired` state the TOTP-verification routes run in. A session with no
/// user yet is left for the handler to reject, so it is permitted here.
fn session_tenant_permits(
    state: Option<&SessionAuthorizationState>,
    url_tenant: &TenantId,
) -> bool {
    match state {
        None => true,
        Some(SessionAuthorizationState::Complete(completed)) => {
            completed.user.tenant == *url_tenant
        }
        Some(SessionAuthorizationState::MFARequired { user }) => user.tenant == *url_tenant,
    }
}

/// Rejects a request whose session user belongs to a tenant other than the one
/// named in the URL.
///
/// Without this a user authenticated in their own tenant could drive the MFA of
/// a same-id user in another tenant (list, delete and enrol credentials) purely
/// by changing the tenant in the path. The check mirrors the one the OIDC
/// session middleware already makes (`oidc/middleware/session_validation.rs`),
/// and lives on the router so no MFA route can omit it.
async fn enforce_session_tenant(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(current_session): Cached<Session>,
    request: Request,
    next: Next,
) -> Result<Response, OperationOutcomeError> {
    let state = session::user::get_authorization_state(&current_session).await?;

    if !session_tenant_permits(state.as_ref(), &tenant) {
        return Err(AuthError::NotLoggedIn.into());
    }

    Ok(next.run(request).await)
}

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _state: Arc<ServerState<Repo, Search, Terminology>>,
) -> Router<Arc<ServerState<Repo, Search, Terminology>>> {
    Router::new()
        .typed_get(admin::admin_get)
        .typed_post(create::create_post)
        .typed_post(delete::mfa_delete_post)
        .typed_get(activate::activate_get)
        .typed_post(activate::activate_post)
        .typed_get(totp_verification::totp_verification_get)
        .typed_post(totp_verification::totp_verification_post)
        .route_layer(axum::middleware::from_fn(enforce_session_tenant))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_n::session::user::AuthorizationStateCompleted;
    use haste_repository::types::user::{AuthMethod, User, UserRole};

    fn user_in(tenant: &str) -> User {
        User {
            id: "shared-id".to_string(),
            tenant: TenantId::new(tenant.to_string()).unwrap(),
            email: Some("user@example.com".to_string()),
            role: UserRole::Owner,
            method: AuthMethod::EmailPassword,
            provider_id: None,
            email_verified: true,
        }
    }

    #[test]
    fn a_completed_session_may_only_act_on_its_own_tenant() {
        let state = SessionAuthorizationState::Complete(AuthorizationStateCompleted {
            user: user_in("my-health"),
        });

        assert!(session_tenant_permits(
            Some(&state),
            &TenantId::new("my-health".to_string()).unwrap()
        ));
        // The cross-tenant account-takeover path: a user logged into their own
        // tenant reaching another tenant's MFA by changing the URL.
        assert!(!session_tenant_permits(
            Some(&state),
            &TenantId::new("victim".to_string()).unwrap()
        ));
    }

    #[test]
    fn an_mfa_required_session_may_only_act_on_its_own_tenant() {
        let state = SessionAuthorizationState::MFARequired {
            user: user_in("my-health"),
        };

        assert!(session_tenant_permits(
            Some(&state),
            &TenantId::new("my-health".to_string()).unwrap()
        ));
        assert!(!session_tenant_permits(
            Some(&state),
            &TenantId::new("victim".to_string()).unwrap()
        ));
    }

    #[test]
    fn a_session_with_no_user_is_left_for_the_handler() {
        assert!(session_tenant_permits(
            None,
            &TenantId::new("my-health".to_string()).unwrap()
        ));
    }
}
