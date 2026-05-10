pub mod providers;
pub mod structs;
pub mod traits;
pub mod validate;

use haste_fhir_model::r4::generated::resources::OperationDefinition;
use haste_fhir_model::r4::generated::types::{Extension, ExtensionValueTypeChoice};

pub const CUSTOM_CODE_EXTENSION_URL: &str = "https://haste.health/Extension/custom-code";
pub const CUSTOM_CODE_TYPE_EXTENSION_URL: &str = "https://haste.health/Extension/custom-code-type";

pub(crate) fn extract_code_from_operation_definition(
    operation: &OperationDefinition,
) -> Option<(&str, &str)> {
    let code_extension = operation.extension.as_ref()?.iter().find(|extension| {
        extension.url == CUSTOM_CODE_EXTENSION_URL
            && extension_value_as_string(extension.as_ref()).is_some()
    })?;

    let code = extension_value_as_string(code_extension.as_ref())?;

    let media_type = code_extension
        .extension
        .as_ref()?
        .iter()
        .find(|extension| extension.url == CUSTOM_CODE_TYPE_EXTENSION_URL)
        .and_then(|extension| extension_value_as_string(extension.as_ref()))?;

    Some((code, media_type))
}

fn extension_value_as_string(extension: &Extension) -> Option<&str> {
    match extension.value.as_ref()? {
        ExtensionValueTypeChoice::String(value) => value.value.as_deref(),
        _ => None,
    }
}
