use oxidized_fhir_model::r4::generated::{
    resources::Resource,
    terminology::IssueType,
    types::{FHIRId, Meta},
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_reflect::MetaValue;

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

pub fn validate_id(id: &str) -> Result<(), OperationOutcomeError> {
    let characters_allowed = ID_CHARACTERS.iter().collect::<String>();
    let re = regex::Regex::new(&format!("^[{}]*$", characters_allowed)).unwrap();
    if !re.is_match(id) {
        Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            format!("ID contains invalid characters: {}", id),
        ))
    } else {
        Ok(())
    }
}

#[derive(OperationOutcomeError)]
pub enum DataTransformError {
    #[error(code = "invalid", diagnostic = "Invalid data: '{arg0}'")]
    InvalidData(String),
    #[error(code = "not-found", diagnostic = "Data not found")]
    NotFound(String),
}

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
        *meta = Some(Box::new(Meta::default()))
    }
    meta.as_mut().map(|meta| {
        meta.versionId = Some(Box::new(FHIRId {
            id: None,
            extension: None,
            value: Some(generate_id(None)),
        }));
    });

    Ok(())
}
