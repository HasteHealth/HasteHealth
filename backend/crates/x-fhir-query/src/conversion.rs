use haste_fhir_model::r4::generated::{
    terminology::IssueType,
    types::{
        FHIRBase64Binary, FHIRBoolean, FHIRCanonical, FHIRDate, FHIRDateTime, FHIRId, FHIRInstant,
        FHIRInteger, FHIRMarkdown, FHIROid, FHIRPositiveInt, FHIRString, FHIRTime, FHIRUnsignedInt,
        FHIRUri, FHIRUrl, FHIRUuid,
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

fn downcast_meta_value<'a, T: 'static>(value: &'a dyn MetaValue) -> Option<&'a T> {
    value.as_any().downcast_ref::<T>()
}

pub fn stringify_meta_value(value: &dyn MetaValue) -> Result<String, OperationOutcomeError> {
    match value.fhir_type() {
        "http://hl7.org/fhirpath/System.String" => downcast_meta_value::<String>(value)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "http://hl7.org/fhirpath/System.String value is missing.".to_string(),
                )
            }),
        "base64Binary" => downcast_meta_value::<FHIRBase64Binary>(value)
            .and_then(|s| s.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "base64Binary value is missing.".to_string(),
                )
            }),
        "decimal" => {
            downcast_meta_value::<haste_fhir_model::r4::generated::types::FHIRDecimal>(value)
                .and_then(|d| d.value.as_ref())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    OperationOutcomeError::fatal(
                        IssueType::INVALID,
                        "decimal value is missing.".to_string(),
                    )
                })
        }

        "boolean" => downcast_meta_value::<FHIRBoolean>(value)
            .and_then(|b| b.value)
            .map(|b| b.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "boolean value is missing.".to_string(),
                )
            }),

        "url" => downcast_meta_value::<FHIRUrl>(value)
            .and_then(|u| u.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "url value is missing.".to_string(),
                )
            }),

        "code" => downcast_meta_value::<haste_fhir_model::r4::generated::types::FHIRCode>(value)
            .and_then(|c| c.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "code value is missing.".to_string(),
                )
            }),

        "string" => downcast_meta_value::<FHIRString>(value)
            .and_then(|s| s.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "string value is missing.".to_string(),
                )
            }),

        "integer" => downcast_meta_value::<FHIRInteger>(value)
            .and_then(|i| i.value)
            .map(|i| i.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "integer value is missing.".to_string(),
                )
            }),

        "uri" => downcast_meta_value::<FHIRUri>(value)
            .and_then(|u| u.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "uri value is missing.".to_string(),
                )
            }),

        "canonical" => downcast_meta_value::<FHIRCanonical>(value)
            .and_then(|c| c.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "canonical value is missing.".to_string(),
                )
            }),

        "markdown" => downcast_meta_value::<FHIRMarkdown>(value)
            .and_then(|m| m.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "markdown value is missing.".to_string(),
                )
            }),

        "id" => downcast_meta_value::<FHIRId>(value)
            .and_then(|id| id.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(IssueType::INVALID, "id value is missing.".to_string())
            }),

        "oid" => downcast_meta_value::<FHIROid>(value)
            .and_then(|o| o.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "oid value is missing.".to_string(),
                )
            }),

        "uuid" => downcast_meta_value::<FHIRUuid>(value)
            .and_then(|u| u.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "uuid value is missing.".to_string(),
                )
            }),

        "unsignedInt" => downcast_meta_value::<FHIRUnsignedInt>(value)
            .and_then(|i| i.value)
            .map(|i| i.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "unsignedInt value is missing.".to_string(),
                )
            }),
        "positiveInt" => downcast_meta_value::<FHIRPositiveInt>(value)
            .and_then(|i| i.value)
            .map(|i| i.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "positiveInt value is missing.".to_string(),
                )
            }),

        "instant" => downcast_meta_value::<FHIRInstant>(value)
            .and_then(|dt| dt.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "instant value is missing.".to_string(),
                )
            }),
        "date" => downcast_meta_value::<FHIRDate>(value)
            .and_then(|dt| dt.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "date value is missing.".to_string(),
                )
            }),
        "time" => downcast_meta_value::<FHIRTime>(value)
            .and_then(|t| t.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "time value is missing.".to_string(),
                )
            }),
        "dateTime" => downcast_meta_value::<FHIRDateTime>(value)
            .and_then(|dt| dt.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::INVALID,
                    "dateTime value is missing.".to_string(),
                )
            }),

        typename => Err(OperationOutcomeError::fatal(
            IssueType::INVALID,
            format!(
                "Unsupported MetaValue type for stringification: '{}'",
                typename
            ),
        )),
    }
}
