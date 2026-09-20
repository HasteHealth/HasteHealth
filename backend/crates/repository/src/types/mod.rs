use haste_fhir_client::request::FHIRRequest;
use std::fmt::Debug;

pub mod authorization_code;
pub mod membership;
pub mod mfa;
pub mod project;
pub mod scope;
pub mod subscription;
pub mod tenant;
pub mod user;

// Defined in haste-jwt so that it can also be carried as an access token claim.
pub use haste_jwt::SupportedFHIRVersions;

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "fhir_method", rename_all = "lowercase")]
pub enum FHIRMethod {
    Create,
    Read,
    Update,
    Delete,
}

impl FHIRMethod {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            FHIRMethod::Create => "create",
            FHIRMethod::Read => "read",
            FHIRMethod::Update => "update",
            FHIRMethod::Delete => "delete",
        }
    }
}

impl TryFrom<&str> for FHIRMethod {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "create" => Ok(FHIRMethod::Create),
            "read" => Ok(FHIRMethod::Read),
            "update" => Ok(FHIRMethod::Update),
            "delete" => Ok(FHIRMethod::Delete),
            _ => Err(format!("Unsupported FHIR method: {value}")),
        }
    }
}

impl TryFrom<&FHIRRequest> for FHIRMethod {
    type Error = String;

    fn try_from(request: &FHIRRequest) -> Result<Self, Self::Error> {
        match request {
            FHIRRequest::Create(_) => Ok(FHIRMethod::Create),
            FHIRRequest::Read(_) => Ok(FHIRMethod::Read),
            FHIRRequest::Update(_) => Ok(FHIRMethod::Update),
            FHIRRequest::Delete(_) => Ok(FHIRMethod::Delete),
            _ => Err("Unsupported FHIR request".to_string()),
        }
    }
}
