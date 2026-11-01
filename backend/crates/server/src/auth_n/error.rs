//! Failures the authentication flows raise in more than one place, defined
//! once so their text and status agree. A failure one route alone reports
//! stays inline in that route.

use haste_fhir_operation_error::derive::OperationOutcomeError;

#[derive(OperationOutcomeError, Debug)]
pub enum AuthError {
    #[error(code = "security", diagnostic = "Invalid CSRF token.")]
    InvalidCsrfToken,
    #[error(code = "security", diagnostic = "User is not logged in.")]
    NotLoggedIn,
    #[error(code = "invalid", diagnostic = "Invalid Authorization header.")]
    InvalidAuthorizationHeader,
    #[fatal(code = "exception", diagnostic = "Failed to set user in session.")]
    SessionWriteFailed(#[from] tower_sessions::session::Error),
    #[fatal(
        code = "invalid",
        diagnostic = "User does not have an email associated."
    )]
    UserWithoutEmail,
    #[error(code = "not-found", diagnostic = "Invalid password reset code.")]
    InvalidPasswordResetCode,
    #[fatal(code = "invalid", diagnostic = "Password reset code has expired.")]
    ExpiredPasswordResetCode,
    #[error(code = "security", diagnostic = "Invalid OTP code.")]
    InvalidOtpCode,
    #[error(code = "security", diagnostic = "Invalid MFA redirect target.")]
    InvalidMfaRedirectTarget,
    #[error(code = "not-found", diagnostic = "User MFA credential not found.")]
    MfaCredentialNotFound,
    #[error(code = "exception", diagnostic = "User MFA credential ID is missing.")]
    MfaCredentialMissingId,
    #[error(
        code = "not-found",
        diagnostic = "The specified identity provider was not found."
    )]
    IdentityProviderNotFound,
    #[error(
        code = "invalid",
        diagnostic = "Identity Provider resource is missing an ID."
    )]
    IdentityProviderMissingId,
}
