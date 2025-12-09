use haste_fhir_client::request::FHIRRequest;
use serde::Serialize;
use std::fmt::{Debug, Display};

pub mod authorization_code;
pub mod membership;
pub mod project;
pub mod scope;
pub mod tenant;
pub mod user;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
#[serde(rename_all = "lowercase")]
pub enum SupportedFHIRVersions {
    R4,
}

impl Display for SupportedFHIRVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedFHIRVersions::R4 => write!(f, "r4"),
        }
    }
}

#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(type_name = "fhir_method", rename_all = "lowercase")]
pub enum FHIRMethod {
    Create,
    Read,
    Update,
    Delete,
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
