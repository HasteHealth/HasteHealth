use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::types::user::User;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

static AUTHORIZATION_STATE_KEY: &str = "user_authorization_state";

#[derive(Deserialize, Serialize)]
pub struct AuthorizationStateCompleted {
    pub user: User,
}

#[derive(Deserialize, Serialize)]
pub enum SessionAuthorizationState {
    Complete(AuthorizationStateCompleted),
    MFARequired { user: User },
    // [TODO] Enforce automatic MFA enrollment for users who have not yet set it up.
    // This will likely be a per tenant setting.
    // MFAEnrollmentRequired { user: User },
}

pub async fn get_completed_authorization_state(
    session: &Session,
) -> Result<AuthorizationStateCompleted, OperationOutcomeError> {
    let authorization_state = get_authorization_state(session).await?;

    match authorization_state {
        Some(SessionAuthorizationState::Complete(completed_state)) => Ok(completed_state),
        _ => Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Authorization state is not complete.".to_string(),
        )),
    }
}

pub async fn get_authorization_state(
    session: &Session,
) -> Result<Option<SessionAuthorizationState>, OperationOutcomeError> {
    let authorization_state = session
        .get::<SessionAuthorizationState>(AUTHORIZATION_STATE_KEY)
        .await
        .map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Session returned an error when retrieving current user.".to_string(),
            )
        })?;

    Ok(authorization_state)
}

pub async fn set_authorization_state(
    session: &Session,
    authorization_state: &SessionAuthorizationState,
) -> Result<(), OperationOutcomeError> {
    session
        .insert(AUTHORIZATION_STATE_KEY, authorization_state)
        .await
        .map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Failed to set user in session.".to_string(),
            )
        })
}

pub async fn clear_authorization_state(session: &Session) -> Result<(), OperationOutcomeError> {
    session
        .remove::<SessionAuthorizationState>(AUTHORIZATION_STATE_KEY)
        .await
        .map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Failed to clear user from session.".to_string(),
            )
        })?;

    Ok(())
}
