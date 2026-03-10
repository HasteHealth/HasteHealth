use crate::r4::generated::resources::RUST_TO_FHIR_TYPE_MAP;
use haste_reflect::MetaValue;

pub mod datetime;
#[cfg(feature = "sqlx")]
pub mod sqlx;
// pub mod terminology;
pub mod generated;

/// Helper function to get the FHIR type from a MetaValue
/// Internally on types we generate hashmap of rust type name to FHIR type, so we can use that to get the FHIR type for a given MetaValue.
pub fn get_fhir_type(value: &dyn MetaValue) -> Option<&'static str> {
    RUST_TO_FHIR_TYPE_MAP.get(value.typename()).map(|s| *s)
}
