use haste_fhir_model::r4::generated::{
    terminology::IssueType,
    types::{
        FHIRBase64Binary, FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDate, FHIRDateTime,
        FHIRDecimal, FHIRId, FHIRInstant, FHIRInteger, FHIRMarkdown, FHIROid, FHIRPositiveInt,
        FHIRString, FHIRTime, FHIRUnsignedInt, FHIRUri, FHIRUrl, FHIRUuid,
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

fn downcast_meta_value<T: 'static>(value: &dyn MetaValue) -> Option<&T> {
    value.as_any().downcast_ref::<T>()
}

/// Converts a [`MetaValue`] into its string representation.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] if:
/// - the value cannot be downcast to the expected FHIR type,
/// - the FHIR primitive value is missing,
/// - the value type is not supported for stringification.
pub fn stringify_meta_value(value: &dyn MetaValue) -> Result<String, OperationOutcomeError> {
    match value.fhir_type() {
        "http://hl7.org/fhirpath/System.String" => downcast_meta_value::<String>(value)
            .cloned()
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::invalid(),
                    "http://hl7.org/fhirpath/System.String value is missing.".to_string(),
                )
            }),
        "base64Binary" => {
            stringify_with::<FHIRBase64Binary, _>(value, "base64Binary", |v| v.value.clone())
        }
        "decimal" => {
            stringify_with::<FHIRDecimal, _>(value, "decimal", |v| v.value.map(|x| x.to_string()))
        }
        "boolean" => {
            stringify_with::<FHIRBoolean, _>(value, "boolean", |v| v.value.map(|x| x.to_string()))
        }
        "url" => stringify_with::<FHIRUrl, _>(value, "url", |v| v.value.clone()),
        "code" => stringify_with::<FHIRCode, _>(value, "code", |v| v.value.clone()),
        "string" => stringify_with::<FHIRString, _>(value, "string", |v| v.value.clone()),
        "integer" => {
            stringify_with::<FHIRInteger, _>(value, "integer", |v| v.value.map(|x| x.to_string()))
        }
        "uri" => stringify_with::<FHIRUri, _>(value, "uri", |v| v.value.clone()),
        "canonical" => stringify_with::<FHIRCanonical, _>(value, "canonical", |v| v.value.clone()),
        "markdown" => stringify_with::<FHIRMarkdown, _>(value, "markdown", |v| v.value.clone()),
        "id" => stringify_with::<FHIRId, _>(value, "id", |v| v.value.clone()),
        "oid" => stringify_with::<FHIROid, _>(value, "oid", |v| v.value.clone()),
        "uuid" => stringify_with::<FHIRUuid, _>(value, "uuid", |v| v.value.clone()),
        "unsignedInt" => stringify_with::<FHIRUnsignedInt, _>(value, "unsignedInt", |v| {
            v.value.map(|x| x.to_string())
        }),
        "positiveInt" => stringify_with::<FHIRPositiveInt, _>(value, "positiveInt", |v| {
            v.value.map(|x| x.to_string())
        }),
        "instant" => stringify_with::<FHIRInstant, _>(value, "instant", |v| {
            v.value.as_ref().map(ToString::to_string)
        }),
        "date" => stringify_with::<FHIRDate, _>(value, "date", |v| {
            v.value.as_ref().map(ToString::to_string)
        }),
        "time" => stringify_with::<FHIRTime, _>(value, "time", |v| {
            v.value.as_ref().map(ToString::to_string)
        }),
        "dateTime" => stringify_with::<FHIRDateTime, _>(value, "dateTime", |v| {
            v.value.as_ref().map(ToString::to_string)
        }),
        typename => Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Unsupported MetaValue type for stringification: '{typename}'"),
        )),
    }
}

fn stringify_with<T, F>(
    value: &dyn MetaValue,
    type_name: &str,
    extractor: F,
) -> Result<String, OperationOutcomeError>
where
    T: MetaValue + 'static,
    F: FnOnce(&T) -> Option<String>,
{
    downcast_meta_value::<T>(value)
        .and_then(extractor)
        .ok_or_else(|| {
            OperationOutcomeError::fatal(
                IssueType::invalid(),
                format!("{type_name} value is missing."),
            )
        })
}
