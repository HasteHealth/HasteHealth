use crate::types::scope::UserId;
use haste_jwt::TenantId;
use sqlx::types::time::OffsetDateTime;

#[derive(sqlx::FromRow, Debug)]
pub struct UserMFACredential {
    pub id: String,
    pub tenant: TenantId,
    pub user_id: String,
    pub credential_type: String,
    pub secret_ciphertext: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub key_id: String,
    pub totp_algorithm: String,
    pub totp_digits: i16,
    pub totp_period: i16,
    pub totp_skew: i16,
    pub is_active: bool,

    pub created_at: OffsetDateTime,
    // pub activated_at: Option<OffsetDateTime>,
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
    pub totp_digits: Option<i16>,
    pub totp_period: Option<i16>,
    pub totp_skew: Option<i16>,
}

// Update model right now is just about activation and deactivation of the MFA credential.
// If we need to update other fields in the future, we can add them here.
pub struct UserMFACredentialUpdate {
    pub user_id: UserId,

    pub activated_at: Option<OffsetDateTime>,
    pub is_active: bool,
}
