use haste_fhir_model::r4::generated::{
    terminology::IssueType,
    types::{ElementDefinition, ElementDefinitionSlicingDiscriminator},
};
use haste_fhir_operation_error::OperationOutcomeError;

/// Various utilities for working with FHIR profiles.

#[allow(dead_code)]
pub fn remove_type_on_path(path: &str) -> &str {
    let first_dot = path.find('.');
    // If first element this would be the entire path as no subfield.
    &path[first_dot.map(|i| i + 1).unwrap_or(path.len())..]
}

#[allow(dead_code)]
/// Because the discriminator path is relative to the element with the discriminator, we need to remove the type on the path to get the correct path to check on the instance data.
pub fn convert_discriminator_to_path(
    discriminator_element: &ElementDefinition,
    discriminator: &ElementDefinitionSlicingDiscriminator,
) -> Result<String, OperationOutcomeError> {
    let Some(discriminator_path) = discriminator.path.value.as_ref() else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Discriminator path is missing".to_string(),
        ));
    };

    if discriminator_path.contains("ofType(")
        || discriminator_path.contains("resolve()")
        || discriminator_path.contains("extension(")
    {
        return Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            format!(
                "Discriminator path '{}' is not supported",
                discriminator_path
            ),
        ));
    }

    let parent_path = remove_type_on_path(discriminator_element.path.value.as_ref().unwrap());
    let path = discriminator_path.replace("$this", "");

    if path.is_empty() {
        Ok(parent_path.to_string())
    } else {
        Ok(format!("{}.{}", parent_path, path))
    }
}
