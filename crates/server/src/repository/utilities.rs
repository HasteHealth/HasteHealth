use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_model::r4::types::{FHIRId, Meta, Resource};
use oxidized_reflect::MetaValue;

// [A-Za-z0-9\-\.]{1,64} See https://hl7.org/fhir/r4/datatypes.html#id
// Can't use _ for compliance.
fn generate_id() -> String {
    nanoid::nanoid!(
        26,
        &[
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '-'
        ]
    )
    .to_string()
}

#[derive(OperationOutcomeError)]
pub enum DataTransformError {
    #[error(code = "invalid", diagnostic = "Invalid data: '{arg0}'")]
    InvalidData(String),
    #[error(code = "not-found", diagnostic = "Data not found")]
    NotFound(String),
}

pub fn set_resource_id(resource: &mut Resource) -> Result<(), OperationOutcomeError> {
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
    *id = Some(generate_id());
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
            value: Some(generate_id()),
        }));
    });

    Ok(())
}
