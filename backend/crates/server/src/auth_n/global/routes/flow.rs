//! The steps shared by sign-up and login: send a code to an address and, once
//! it is verified, put the person in a tenant. An address with no account is
//! offered a tenant of its own, with an id and a password to choose. An
//! invitation (a user a tenant created for the address) must be accepted
//! before it can be entered.

use crate::{
    auth_n::{
        email::send_email_code,
        global::{
            bot_check::{self, Verdict},
            email_code::{self, CODE_VALID_FOR, Purpose, VerifiedEmail},
        },
        invitations,
        mfa::routes::totp_verification::totp_verification_route,
        oidc::{
            hardcoded_clients::admin_app, routes::route_string::tenant_route_string,
            utilities::check_password_strength,
        },
        session::{self, user::SessionAuthorizationState},
    },
    services::ServerState,
    tenants::{
        CreateTenantOutput, create_tenant, read_tenant, suggested_tenant_id,
        suggested_tenant_id_variant, validate_tenant_id,
    },
    ui::pages::global_auth::{
        self, LOGIN_ROUTE, SIGNUP_ROUTE, TenantChoice, code_entry_html, create_workspace_html,
        email_form_html,
    },
};
use axum::response::{IntoResponse, Redirect, Response};
use axum_client_ip::ClientIp;
use email_address::EmailAddress;
use haste_fhir_model::r4::generated::{
    resources::User as UserResource,
    terminology::{IssueType, UserRole},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, scopes::Scopes};
use haste_rate_limit::RateLimitError;
use haste_repository::{
    Repository,
    admin::{ProjectModelAdmin, SystemAdmin, TenantModelAdmin},
    types::{
        scope::{ClientId, CreateScope, UserId},
        user::{AuthMethod, User, UserSearchClauses},
    },
};
use serde::Deserialize;
use std::{collections::HashSet, net::IpAddr, str::FromStr};
use tower_sessions::Session;
use url::form_urlencoded;

const CODES_PER_EMAIL_PER_HOUR: i32 = 5;
/// Across all addresses.
const CODES_PER_IP_PER_HOUR: i32 = 20;
const HOUR_SECONDS: i32 = 60 * 60;

/// Outer error: the server failed. Inner error: the request was refused, with
/// the message to show.
type Refusable<T> = Result<Result<T, String>, OperationOutcomeError>;

#[derive(Deserialize)]
pub struct EmailForm {
    pub csrf_token: String,
    pub email: String,
    /// Honeypot. See [`bot_check`].
    pub website: Option<String>,
    #[serde(rename = "cf-turnstile-response")]
    pub turnstile: Option<String>,
}

/// `None` behind an unconfigured proxy; the per-IP limit is then not applied.
pub fn client_ip(ip: Result<ClientIp, axum_client_ip::Rejection>) -> Option<IpAddr> {
    ip.ok().map(|ClientIp(ip)| ip)
}

pub fn check_csrf(expected: &str, submitted: &str) -> Result<(), OperationOutcomeError> {
    if expected == submitted {
        Ok(())
    } else {
        Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Invalid CSRF Token".to_string(),
        ))
    }
}

pub fn turnstile_site_key<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
) -> Option<&str> {
    state
        .config
        .signup
        .turnstile
        .as_ref()
        .map(|turnstile| turnstile.site_key.as_str())
}

pub fn start_over_route(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::Signup => SIGNUP_ROUTE,
        Purpose::Login => LOGIN_ROUTE,
    }
}

/// Spends one point of an hourly limit.
async fn spend_rate_limit<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    key: &str,
    max: i32,
) -> Refusable<()> {
    match state.rate_limit.check(key, max, 1, HOUR_SECONDS).await {
        Ok(_) => Ok(Ok(())),
        Err(RateLimitError::Exceeded) => Ok(Err(
            "Too many codes requested. Wait a while, then try again.".to_string(),
        )),
        Err(RateLimitError::Error(error)) => {
            tracing::error!(error, key, "rate limit check failed");
            Err(OperationOutcomeError::fatal(
                IssueType::exception(),
                "Could not check the request limit.".to_string(),
            ))
        }
    }
}

/// Emails a fresh code for `email`, within the limits.
async fn send_code<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    email: &EmailAddress,
    purpose: Purpose,
    ip: Option<IpAddr>,
) -> Refusable<()> {
    if let Some(ip) = ip
        && let Err(message) =
            spend_rate_limit(state, &format!("email_code:ip:{ip}"), CODES_PER_IP_PER_HOUR).await?
    {
        return Ok(Err(message));
    }

    if let Err(message) = spend_rate_limit(
        state,
        &format!("email_code:email:{}", email.as_str().to_lowercase()),
        CODES_PER_EMAIL_PER_HOUR,
    )
    .await?
    {
        return Ok(Err(message));
    }

    let code = email_code::start(session, email.as_str(), purpose).await?;

    send_email_code(
        &state.config.email,
        &state.config.api_uri,
        email,
        &code,
        CODE_VALID_FOR,
    )
    .await?;

    Ok(Ok(()))
}

/// Handles the email form for either purpose.
pub async fn begin<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    csrf_token: &str,
    ip: Option<IpAddr>,
    purpose: Purpose,
    form: EmailForm,
) -> Result<Response, OperationOutcomeError> {
    check_csrf(csrf_token, &form.csrf_token)?;

    let form_with = |error: &str| {
        email_form_html(purpose, csrf_token, turnstile_site_key(state), Some(error)).into_response()
    };

    let Ok(email) = EmailAddress::from_str(form.email.trim()) else {
        return Ok(form_with("Enter a valid email address."));
    };

    match bot_check::check(
        &state.config.signup,
        form.website.as_deref(),
        form.turnstile.as_deref(),
        ip,
    )
    .await?
    {
        // A bot gets the same page as everyone else, but no code is sent or
        // pending, so nothing it submits can verify.
        Verdict::Bot(reason) => {
            tracing::warn!(reason, ?ip, "email code request looked automated");
        }
        Verdict::Human => {
            if let Err(message) = send_code(state, session, &email, purpose, ip).await? {
                return Ok(form_with(&message));
            }
        }
    }

    Ok(code_entry_html(
        csrf_token,
        email.as_str(),
        start_over_route(purpose),
        None,
        None,
    )
    .into_response())
}

/// Sends another code to the address pending in this session.
pub async fn resend<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    csrf_token: &str,
    ip: Option<IpAddr>,
) -> Result<Response, OperationOutcomeError> {
    let Some(pending) = email_code::pending(session).await? else {
        return Ok(Redirect::to(LOGIN_ROUTE).into_response());
    };

    let Ok(email) = EmailAddress::from_str(&pending.email) else {
        return Ok(Redirect::to(start_over_route(pending.purpose)).into_response());
    };

    let sent = send_code(state, session, &email, pending.purpose, ip).await?;

    Ok(code_entry_html(
        csrf_token,
        email.as_str(),
        start_over_route(pending.purpose),
        sent.is_ok().then_some("We sent a new code."),
        sent.as_ref().err().map(String::as_str),
    )
    .into_response())
}

/// One user per tenant for an email address. Users who sign in through an
/// identity provider are left to that provider.
async fn users_for_email<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    email: &str,
) -> Result<Vec<User>, OperationOutcomeError> {
    let users = SystemAdmin::<User, UserSearchClauses>::search(
        state.repo.as_ref(),
        &UserSearchClauses {
            email: Some(email.to_string()),
            role: None,
            method: Some(AuthMethod::EmailPassword),
        },
    )
    .await?;

    let mut seen = HashSet::new();
    Ok(users
        .into_iter()
        .filter(|user| seen.insert(user.tenant.clone()))
        .collect())
}

/// The email-password user `tenant` holds for `email`.
async fn user_in_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    email: &str,
) -> Result<User, OperationOutcomeError> {
    TenantModelAdmin::<_, User, _, _, String>::search(
        state.repo.as_ref(),
        tenant,
        &UserSearchClauses {
            email: Some(email.to_string()),
            role: None,
            method: Some(AuthMethod::EmailPassword),
        },
    )
    .await?
    .into_iter()
    .next()
    .ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::not_found(),
            "No user with the verified email address in that tenant.".to_string(),
        )
    })
}

/// What follows a verified address: its only tenant, the form to create one,
/// or a chooser listing its tenants and invitations.
pub async fn finish<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    csrf_token: &str,
    verified: VerifiedEmail,
) -> Result<Response, OperationOutcomeError> {
    let mut users = users_for_email(state, &verified.email).await?;

    match users.as_slice() {
        [] => workspace_form(state, csrf_token, &verified, None, None).await,
        [user] if user.email_verified => {
            email_code::clear_verified(session).await?;
            login_user(state, session, users.remove(0), None).await
        }
        _ => {
            let mut choices = Vec::with_capacity(users.len());
            for user in &users {
                let name = read_tenant(state, &user.tenant)
                    .await
                    .ok()
                    .and_then(|tenant| tenant.display_name);
                choices.push(TenantChoice {
                    id: user.tenant.as_ref().to_string(),
                    name,
                    href: tenant_choice_route(&user.tenant),
                    invited: !user.email_verified,
                });
            }

            Ok(
                global_auth::tenant_chooser_html(csrf_token, &verified.email, &choices)
                    .into_response(),
            )
        }
    }
}

fn tenant_choice_route(tenant: &TenantId) -> String {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("tenant", tenant.as_ref())
        .finish();

    format!("{}?{query}", global_auth::TENANT_ROUTE)
}

/// Logs the verified address into `tenant`. An unaccepted invitation goes
/// back to the chooser instead.
pub async fn choose_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    csrf_token: &str,
    verified: VerifiedEmail,
    tenant: &TenantId,
) -> Result<Response, OperationOutcomeError> {
    let user = user_in_tenant(state, tenant, &verified.email).await?;

    if !user.email_verified {
        return finish(state, session, csrf_token, verified).await;
    }

    email_code::clear_verified(session).await?;
    login_user(state, session, user, None).await
}

/// Accepts `tenant`'s invitation for the verified address and logs it in.
pub async fn accept_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    verified: VerifiedEmail,
    tenant: &TenantId,
) -> Result<Response, OperationOutcomeError> {
    let user = user_in_tenant(state, tenant, &verified.email).await?;
    let user = invitations::accept(state, user).await?;

    email_code::clear_verified(session).await?;
    login_user(state, session, user, None).await
}

/// Declines `tenant`'s invitation for the verified address, then shows what
/// is left.
pub async fn decline_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    csrf_token: &str,
    verified: VerifiedEmail,
    tenant: &TenantId,
) -> Result<Response, OperationOutcomeError> {
    let user = user_in_tenant(state, tenant, &verified.email).await?;
    invitations::decline(state, &user).await?;

    finish(state, session, csrf_token, verified).await
}

/// The "name your workspace" page. `tenant_id` defaults to a free suggestion
/// for the address.
pub async fn workspace_form<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    csrf_token: &str,
    verified: &VerifiedEmail,
    tenant_id: Option<&str>,
    error: Option<&str>,
) -> Result<Response, OperationOutcomeError> {
    let tenant_id = match tenant_id {
        Some(tenant_id) => tenant_id.to_string(),
        None => free_tenant_id(state, &verified.email).await,
    };

    Ok(create_workspace_html(
        csrf_token,
        &verified.email,
        verified.purpose,
        &tenant_id,
        error,
    )
    .into_response())
}

/// The suggestion for an address, suffixed when it is already taken.
async fn free_tenant_id<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    email: &str,
) -> String {
    let candidate = suggested_tenant_id(email);

    if tenant_exists(state, &TenantId::new(candidate.clone())).await {
        suggested_tenant_id_variant(&candidate)
    } else {
        candidate
    }
}

async fn tenant_exists<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
) -> bool {
    read_tenant(state, tenant).await.is_ok()
}

fn first_diagnostic(error: &OperationOutcomeError) -> String {
    error
        .outcome()
        .issue
        .first()
        .and_then(|issue| issue.diagnostics.as_ref())
        .and_then(|diagnostics| diagnostics.value.clone())
        .unwrap_or_default()
}

fn is_duplicate(error: &OperationOutcomeError) -> bool {
    error
        .outcome()
        .issue
        .first()
        .is_some_and(|issue| issue.code == IssueType::duplicate())
}

/// Creates the tenant the verified address asked for and logs it in. A
/// refused id or password brings the form back with the reason.
pub async fn create_workspace<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    csrf_token: &str,
    verified: VerifiedEmail,
    requested_id: &str,
    password: &str,
) -> Result<Response, OperationOutcomeError> {
    // A double submit must not create two tenants for one address.
    // Invitations don't count.
    if let Some(existing) = users_for_email(state, &verified.email)
        .await?
        .into_iter()
        .find(|user| user.email_verified)
    {
        email_code::clear_verified(session).await?;
        return login_user(state, session, existing, None).await;
    }

    let requested_id = requested_id.trim().to_ascii_lowercase();

    let created = match new_tenant(state, &verified.email, &requested_id, password).await? {
        Ok(created) => created,
        Err(message) => {
            return workspace_form(
                state,
                csrf_token,
                &verified,
                Some(&requested_id),
                Some(&message),
            )
            .await;
        }
    };

    tracing::info!(tenant = %created.tenant.id, "tenant created through sign-up");

    preapprove_admin_app(state, &created.tenant.id, &created.owner.id).await?;
    email_code::clear_verified(session).await?;

    // Straight to the admin app: the project page reads a search index that
    // may not have caught up with a tenant created moments ago.
    let admin_app = admin_app::redirect_url(&state.config, &created.tenant.id, &ProjectId::System);
    login_user(state, session, created.owner, admin_app).await
}

/// Checks `id` and `password`, then creates the tenant with `email` as owner.
async fn new_tenant<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    email: &str,
    id: &str,
    password: &str,
) -> Refusable<CreateTenantOutput> {
    let tenant_id = match validate_tenant_id(id) {
        Ok(tenant_id) => tenant_id,
        Err(error) => return Ok(Err(first_diagnostic(&error))),
    };

    let taken = || format!("'{id}' is taken. Choose another tenant id.");
    if tenant_exists(state, &tenant_id).await {
        return Ok(Err(taken()));
    }

    if password.is_empty() {
        return Ok(Err("Choose a password.".to_string()));
    }

    if let Err(error) = check_password_strength(password, email) {
        return Ok(Err(format!(
            "That password is too easy to guess. {}",
            first_diagnostic(&error)
        )));
    }

    let owner = UserResource {
        role: UserRole::owner(),
        email: Some(Box::new(FHIRString {
            value: Some(email.to_string()),
            ..Default::default()
        })),
        ..Default::default()
    };

    match create_tenant(
        state,
        Some(id.to_string()),
        &haste_subscription::DEFAULT_TIER,
        owner,
        Some(password),
    )
    .await
    {
        Ok(created) => Ok(Ok(created)),
        // Lost a race for the id since the check above.
        Err(error) if is_duplicate(&error) => Ok(Err(taken())),
        Err(error) => Err(error),
    }
}

/// Approves the admin app's scopes for a new owner, who signed up for exactly
/// that, so no consent page is shown.
async fn preapprove_admin_app<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    user_id: &str,
) -> Result<(), OperationOutcomeError> {
    let Some(app) = admin_app::get_admin_app(&state.config) else {
        return Ok(());
    };
    let (Some(client_id), Some(scope)) = (app.id, app.scope.and_then(|scope| scope.value)) else {
        return Ok(());
    };

    ProjectModelAdmin::create(
        state.repo.as_ref(),
        tenant,
        &ProjectId::System,
        CreateScope {
            client: ClientId::new(client_id),
            user_: UserId::new(user_id.to_string()),
            scope: Scopes::try_from(scope.as_str())?,
        },
    )
    .await?;

    Ok(())
}

/// Authenticates the session for `user` and redirects to `destination`, or to
/// the tenant's project page without one. A user with MFA enrolled goes
/// through TOTP and then the project page: that route only returns to a
/// local path.
pub async fn login_user<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    session: &Session,
    user: User,
    destination: Option<String>,
) -> Result<Response, OperationOutcomeError> {
    let tenant = user.tenant.clone();

    session::user::rotate_session_id(session).await?;
    session::user::set_initial_authorization_state(state.repo.as_ref(), session, user).await?;

    let project_select = tenant_route_string(&tenant)
        .join("auth/interactions/project-select")
        .to_string_lossy()
        .into_owned();

    let target = match session::user::get_authorization_state(session).await? {
        Some(SessionAuthorizationState::MFARequired { .. }) => {
            totp_verification_route(&tenant, &project_select)
        }
        _ => destination.unwrap_or(project_select),
    };

    Ok(Redirect::to(&target).into_response())
}
