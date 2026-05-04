use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

pub enum PluginCodeType {
    JavaScript,
    TypeScript,
}

impl TryFrom<&str> for PluginCodeType {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "typescript" | "text/typescript" | "application/typescript" => {
                Ok(PluginCodeType::TypeScript)
            }
            "javascript" | "text/javascript" | "application/javascript" => {
                Ok(PluginCodeType::JavaScript)
            }
            _ => Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                format!("Unsupported custom-code media-type: {value}"),
            )),
        }
    }
}
