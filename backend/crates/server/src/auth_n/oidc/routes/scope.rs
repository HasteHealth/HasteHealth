use crate::auth_n::constant_time;
use crate::{
    auth_n::{
        oidc::{
            error::{OIDCError, OIDCErrorCode},
            extract::client_app::find_client_app,
            routes::route_string::oidc_route_string,
            utilities::is_valid_redirect_url,
        },
        session,
    },
    extract::{
        csrf_token::CSRFToken,
        path_tenant::{ProjectIdentifier, TenantIdentifier},
    },
    services::ServerState,
};
use axum::{
    extract::{OriginalUri, State},
    response::{IntoResponse, Response},
};
use axum_extra::{
    extract::{Cached, Form},
    routing::TypedPath,
};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::scopes::{OIDCScope, Scope, Scopes, SmartScope};
use haste_repository::{
    Repository,
    admin::ProjectModelAdmin,
    types::scope::{ClientId, CreateScope, UserId},
};
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/scope")]
pub struct ScopePost;

#[derive(Deserialize, Debug)]
pub struct ScopeForm {
    pub csrf_token: String,
    pub client_id: String,
    pub response_type: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    /// On the approval page, the scopes the app requested. Posted back, the
    /// scopes the user approved: one `scope` field per checked scope, or a
    /// single space-separated value.
    #[serde(default, deserialize_with = "scopes_from_values")]
    pub scope: Scopes,
    pub redirect_uri: String,
    /// SMART EHR launch: the launch value, carried back to authorize.
    pub launch: Option<String>,
    /// SMART App Launch: the FHIR server the app will call, carried back to
    /// authorize so it is checked there.
    pub aud: Option<String>,
    pub accept: Option<String>,
}

/// Every value of a repeated `scope` field, joined and parsed as one scope
/// string.
fn scopes_from_values<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Scopes, D::Error> {
    let values = Vec::<String>::deserialize(deserializer)?;
    Scopes::try_from(values.join(" ").as_str()).map_err(serde::de::Error::custom)
}

/// Whether the approval page keeps `scope` checked. These scopes shape the
/// flow rather than grant access to data: `openid` asks for the id_token, and
/// the launch scopes for the context the app runs in (`launch` carries an EHR
/// launch's patient and encounter). Every other scope the user may deselect.
pub fn is_fixed_scope(scope: &Scope) -> bool {
    matches!(
        scope,
        Scope::OIDC(OIDCScope::OpenId)
            | Scope::SMART(SmartScope::LaunchSystem(_) | SmartScope::LaunchType(_))
    )
}

/// Every requested scope must be within what the client is configured to
/// request. The configured scopes are a ceiling, not a list of exact strings:
/// `patient/*.rs` lets a client request `patient/Medication.rs`, and
/// `patient/Medication.rs` lets it request `patient/Medication.s` (see
/// [`Scopes::covers_scope`]).
pub fn verify_requested_scope_is_subset(
    requested: &Scopes,
    allowed: &Scopes,
) -> Result<(), OIDCError> {
    for scope in requested.0.iter() {
        if !allowed.covers_scope(scope) {
            return Err(OIDCError::new(
                OIDCErrorCode::InvalidScope,
                Some(format!(
                    "Requested scope '{}' is not allowed. Check client configuration for what scopes are allowed.",
                    String::from(scope.clone())
                )),
                None,
            ));
        }
    }
    Ok(())
}

pub async fn scope_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: ScopePost,
    _uri: OriginalUri,
    CSRFToken(csrf_token): CSRFToken,
    State(app_state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Form(scope_data): Form<ScopeForm>,
) -> Result<Response, OIDCError> {
    // The client comes from the approval form, which repeats `scope` and so
    // is not run through the OIDC parameter layer that authorize uses.
    let client_app = find_client_app(
        &app_state,
        tenant.clone(),
        project.clone(),
        scope_data.client_id.clone(),
    )
    .await?;

    // The form's redirect URI is only redirected to once it is known to be
    // registered for the client (RFC 6749 §4.1.2.1); errors on a request that
    // names some other URI are answered in the response instead.
    let error_redirect_uri = is_valid_redirect_url(&scope_data.redirect_uri, &client_app)
        .then(|| scope_data.redirect_uri.clone());

    if !constant_time::str_eq(&csrf_token, &scope_data.csrf_token) {
        return Err(OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("Invalid CSRF Token.".to_string()),
            error_redirect_uri,
        ));
    }

    let completed_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to retrieve user from session.".to_string()),
                error_redirect_uri.clone(),
            )
        })?;

    // Approving none of the scopes is a denial. Otherwise the approved subset
    // is stored and authorize runs again with it, so the code and the token
    // carry only what the user approved.
    if let Some("on") = scope_data.accept.as_deref()
        && !scope_data.scope.0.is_empty()
    {
        let client_scope = client_app
            .scope
            .as_ref()
            .and_then(|s| s.value.clone())
            .unwrap_or_default();

        let allowed = Scopes::try_from(client_scope.as_str()).map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::InvalidScope,
                Some("Client application has an invalid scope configured.".to_string()),
                error_redirect_uri.clone(),
            )
        })?;
        verify_requested_scope_is_subset(&scope_data.scope, &allowed)?;

        let client_id = client_app.id.clone().ok_or_else(|| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to retrieve client ID.".to_string()),
                None,
            )
        })?;

        ProjectModelAdmin::create(
            &*app_state.repo,
            &tenant,
            &project,
            CreateScope {
                client: ClientId::new(client_id.clone()),
                user_: UserId::new(completed_auth_state.user.id),
                scope: scope_data.scope.clone(),
            },
        )
        .await
        .map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to create scope authorization.".to_string()),
                error_redirect_uri.clone(),
            )
        })?;

        // Back to authorize with the same request, but the approved scopes,
        // which now match the stored ones. Encoded: the scope has spaces, and
        // state, redirect_uri and launch are the client's own.
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        query
            .append_pair("client_id", client_id.as_str())
            .append_pair("response_type", scope_data.response_type.as_str())
            .append_pair("state", scope_data.state.as_str())
            .append_pair("code_challenge", scope_data.code_challenge.as_str())
            .append_pair(
                "code_challenge_method",
                scope_data.code_challenge_method.as_str(),
            )
            .append_pair("scope", String::from(scope_data.scope).as_str())
            .append_pair("redirect_uri", scope_data.redirect_uri.as_str());
        if let Some(launch) = scope_data.launch.as_deref() {
            query.append_pair("launch", launch);
        }
        if let Some(aud) = scope_data.aud.as_deref() {
            query.append_pair("aud", aud);
        }

        let authorization_route = oidc_route_string(&tenant, &project, "auth/authorize")
            .to_str()
            .expect("Could not create authorize route.")
            .to_string()
            + "?"
            + query.finish().as_str();
        let redirect = axum::response::Redirect::to(&authorization_route);
        Ok(redirect.into_response())
    } else {
        Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("User did not accept the requested scopes.".to_string()),
            error_redirect_uri,
        ))
    }
}
