use crate::types::scope::UserId;
use haste_jwt::TenantId;
use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct UserMFACredential {
    pub id: String,
    pub tenant: TenantId,
    pub user_id: String,
    pub credential_type: String,
    pub secret_ciphertext: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub key_id: String,
    pub totp_algorithm: String,
    pub totp_digits: u32,
    pub totp_period: u32,
    pub totp_skew: u32,
    pub created_at: chrono::NaiveDateTime,
    pub activated_at: Option<chrono::NaiveDateTime>,
    pub is_active: bool,
}

pub struct UserMFASearchClaims {
    pub tenant: TenantId,
    pub user_id: UserId,
    pub is_active: Option<bool>,
}

pub enum MFACredentialType {
    TOTP,
}

impl From<MFACredentialType> for &str {
    fn from(credential_type: MFACredentialType) -> Self {
        match credential_type {
            MFACredentialType::TOTP => "totp",
        }
    }
}

pub struct UserMFACredentialCreate {
    pub user_id: UserId,
    pub credential_type: MFACredentialType,
    pub secret_ciphertext: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub key_id: String,
    pub totp_algorithm: Option<String>,
    pub totp_digits: Option<u32>,
    pub totp_period: Option<u32>,
    pub totp_skew: Option<u32>,
}

// Update model right now is just about activation and deactivation of the MFA credential.
// If we need to update other fields in the future, we can add them here.
pub struct UserMFACredentialUpdate {
    pub user_id: UserId,

    pub activated_at: Option<chrono::NaiveDateTime>,
    pub is_active: Option<bool>,
}
