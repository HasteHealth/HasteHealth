use std::sync::LazyLock;

use haste_fhir_model::r4::generated::{
    resources::Resource,
    terminology::IssueType,
    types::{FHIRId, Meta},
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_reflect::MetaValue;

static ID_CHARACTERS: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '-',
];

// [A-Za-z0-9\-\.]{1,64} See https://hl7.org/fhir/r4/datatypes.html#id
// Can't use _ for compliance.
pub fn generate_id(len: Option<usize>) -> String {
    let len = len.unwrap_or(26);
    nanoid::nanoid!(len, ID_CHARACTERS).to_string()
}

static ID_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    let characters_allowed = ID_CHARACTERS.iter().collect::<String>();
    regex::Regex::new(&format!("^[{characters_allowed}]*$"))
        .expect("ID_CHARACTERS should produce a valid regex")
});

/// Validates a FHIR resource ID.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] if `id` contains characters that are
/// not permitted in a FHIR resource ID.
pub fn validate_id(id: &str) -> Result<(), OperationOutcomeError> {
    if ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("ID contains invalid characters: {id}"),
        ))
    }
}

#[derive(OperationOutcomeError)]
pub enum DataTransformError {
    #[error(code = "invalid", diagnostic = "Invalid data: '{arg0}'")]
    InvalidData(String),
    #[error(code = "not-found", diagnostic = "Data not found")]
    NotFound(String),
}

/// Sets the ID of a FHIR resource.
///
/// If `id_` is `Some`, that value is assigned as the resource ID. Otherwise, a
/// new ID is generated and assigned.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] if the resource does not contain an `id`
/// field or if the `id` field is not of type [`Option<String>`].
pub fn set_resource_id(
    resource: &mut Resource,
    id_: Option<String>,
) -> Result<(), OperationOutcomeError> {
    let id: &mut dyn std::any::Any =
        resource
            .get_field_mut("id")
            .ok_or(DataTransformError::InvalidData(
                "Missing 'id' field".to_string(),
            ))?;
    let id: &mut Option<String> =
        id.downcast_mut::<Option<String>>()
            .ok_or(DataTransformError::InvalidData(
                "Invalid 'id' field".to_string(),
            ))?;
    *id = Some(id_.unwrap_or_else(|| generate_id(None)));
    Ok(())
}

/// Sets the version ID of a FHIR resource.
///
/// If the resource does not have a `meta` element, one is created. The
/// `versionId` field is then populated with a newly generated ID.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] if the resource does not contain a
/// `meta` field or if the `meta` field is not of type [`Option<Box<Meta>>`].
pub fn set_version_id(resource: &mut Resource) -> Result<(), OperationOutcomeError> {
    let meta: &mut dyn std::any::Any =
        resource
            .get_field_mut("meta")
            .ok_or(DataTransformError::InvalidData(
                "Missing 'meta' field".to_string(),
            ))?;
    let meta: &mut Option<Box<Meta>> =
        meta.downcast_mut::<Option<Box<Meta>>>()
            .ok_or(DataTransformError::InvalidData(
                "Invalid 'meta' field".to_string(),
            ))?;

    if meta.is_none() {
        *meta = Some(Box::new(Meta::default()));
    }
    if let Some(meta) = meta.as_mut() {
        meta.versionId = Some(Box::new(FHIRId {
            id: None,
            extension: None,
            value: Some(generate_id(None)),
        }));
    }

    Ok(())
}
