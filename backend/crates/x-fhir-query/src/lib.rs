use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, FPEngine};
use haste_reflect::MetaValue;
use std::sync::Arc;

pub async fn evaluation<'a, 'b>(
    path: &str,
    values: Vec<&'a dyn MetaValue>,
    config: Arc<Config<'b>>,
) -> Result<Option<String>, OperationOutcomeError>
where
    'a: 'b,
{
    let engine = FPEngine::new();
    let result = engine
        .evaluate_with_config(path, values, config)
        .await
        .map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                format!("FHIRPath evaluation error: {}", e),
            )
        })?;

    let collection = result.iter().map(|v| v.to_string()).collect::<Vec<_>>();

    Ok(None)
}
