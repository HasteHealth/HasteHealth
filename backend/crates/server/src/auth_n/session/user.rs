use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_repository::types::user::User;
use tower_sessions::Session;

static USER_KEY: &str = "auth_user";

pub async fn get_user(session: Session) -> Result<Option<User>, OperationOutcomeError> {
    let user = session.get::<User>(USER_KEY).await.map_err(|_e| {
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Session returned an error when retrieving current user.".to_string(),
        )
    })?;

    Ok(user)
}

pub async fn set_user(session: Session, user: &User) -> Result<(), OperationOutcomeError> {
    session.insert(USER_KEY, user).await.map_err(|_e| {
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Failed to set user in session.".to_string(),
        )
    })
}

pub async fn clear_user(session: Session) -> Result<(), OperationOutcomeError> {
    session.remove::<User>(USER_KEY).await.map_err(|_e| {
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Failed to clear user from session.".to_string(),
        )
    })?;

    Ok(())
}
