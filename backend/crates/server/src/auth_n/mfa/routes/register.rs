use axum::response::{IntoResponse as _, Response};
use axum_extra::routing::TypedPath;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use maud::html;

#[derive(TypedPath)]
#[typed_path("/register")]
pub struct MFARegisterGET;

// Sends a QR code image for a MFA registration request. The QR code is generated based on the user's email and a secret key,
// which is used for TOTP (Time-based One-Time Password) authentication.
// To activate the MR user most enter code which flips the db table to active.
pub async fn register_get(_: MFARegisterGET) -> Result<Response, OperationOutcomeError> {
    let secret = totp_rs::Secret::default().to_bytes().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate secret for MFA".to_string(),
        )
    })?;
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("Haste Health".to_string()),
        "haste-health@github.com".to_string(),
    )
    .map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate TOTP for MFA".to_string(),
        )
    })?;

    let qr_code = totp.get_qr_base64().map_err(|e| {
        tracing::error!(error = ?e);

        OperationOutcomeError::error(
            IssueType::Exception(None),
            "Could not generate QR code for MFA".to_string(),
        )
    })?;

    Ok(html! {
        img src=(format!("data:image/png;base64,{}", qr_code)) {}
    }
    .into_response())
}

#[derive(TypedPath)]
#[typed_path("/register")]
pub struct MFARegisterPOST;

pub async fn register_post(_: MFARegisterPOST) -> Result<Response, OperationOutcomeError> {
    Ok((axum::http::StatusCode::OK, "MFA registration successful").into_response())
}
