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
    match value.typename() {
        "http://hl7.org/fhirpath/System.String" => downcast_meta_value::<String>(value)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "http://hl7.org/fhirpath/System.String value is missing.".to_string(),
                )
            }),
        "FHIRBase64Binary" => downcast_meta_value::<FHIRBase64Binary>(value)
            .and_then(|s| s.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRBase64Binary value is missing.".to_string(),
                )
            }),
        "FHIRDecimal" => {
            downcast_meta_value::<haste_fhir_model::r4::generated::types::FHIRDecimal>(value)
                .and_then(|d| d.value.as_ref())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        "FHIRDecimal value is missing.".to_string(),
                    )
                })
        }

        "FHIRBoolean" => downcast_meta_value::<FHIRBoolean>(value)
            .and_then(|b| b.value)
            .map(|b| b.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRBoolean value is missing.".to_string(),
                )
            }),

        "FHIRUrl" => downcast_meta_value::<FHIRUrl>(value)
            .and_then(|u| u.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRUrl value is missing.".to_string(),
                )
            }),

        "FHIRCode" => {
            downcast_meta_value::<haste_fhir_model::r4::generated::types::FHIRCode>(value)
                .and_then(|c| c.value.as_ref())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        "FHIRCode value is missing.".to_string(),
                    )
                })
        }

        "FHIRString" => downcast_meta_value::<FHIRString>(value)
            .and_then(|s| s.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRString value is missing.".to_string(),
                )
            }),

        "FHIRInteger" => downcast_meta_value::<FHIRInteger>(value)
            .and_then(|i| i.value)
            .map(|i| i.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRInteger value is missing.".to_string(),
                )
            }),

        "FHIRUri" => downcast_meta_value::<FHIRUri>(value)
            .and_then(|u| u.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRUri value is missing.".to_string(),
                )
            }),

        "FHIRCanonical" => downcast_meta_value::<FHIRCanonical>(value)
            .and_then(|c| c.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRCanonical value is missing.".to_string(),
                )
            }),

        "FHIRMarkdown" => downcast_meta_value::<FHIRMarkdown>(value)
            .and_then(|m| m.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRMarkdown value is missing.".to_string(),
                )
            }),

        "FHIRId" => downcast_meta_value::<FHIRId>(value)
            .and_then(|id| id.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRId value is missing.".to_string(),
                )
            }),

        "FHIROid" => downcast_meta_value::<FHIROid>(value)
            .and_then(|o| o.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIROid value is missing.".to_string(),
                )
            }),

        "FHIRUuid" => downcast_meta_value::<FHIRUuid>(value)
            .and_then(|u| u.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRUuid value is missing.".to_string(),
                )
            }),

        "FHIRUnsignedInt" => downcast_meta_value::<FHIRUnsignedInt>(value)
            .and_then(|i| i.value)
            .map(|i| i.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRUnsignedInt value is missing.".to_string(),
                )
            }),
        "FHIRPositiveInt" => downcast_meta_value::<FHIRPositiveInt>(value)
            .and_then(|i| i.value)
            .map(|i| i.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRPositiveInt value is missing.".to_string(),
                )
            }),

        "FHIRInstant" => downcast_meta_value::<FHIRInstant>(value)
            .and_then(|dt| dt.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRInstant value is missing.".to_string(),
                )
            }),
        "FHIRDate" => downcast_meta_value::<FHIRDate>(value)
            .and_then(|dt| dt.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRDate value is missing.".to_string(),
                )
            }),
        "FHIRTime" => downcast_meta_value::<FHIRTime>(value)
            .and_then(|t| t.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRTime value is missing.".to_string(),
                )
            }),
        "FHIRDateTime" => downcast_meta_value::<FHIRDateTime>(value)
            .and_then(|dt| dt.value.as_ref())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "FHIRDateTime value is missing.".to_string(),
                )
            }),

        typename => Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            format!(
                "Unsupported MetaValue type for stringification: '{}'",
                typename
            ),
        )),
    }
}
