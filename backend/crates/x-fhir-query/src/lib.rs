use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, FPEngine};
use haste_reflect::MetaValue;
use regex::Regex;
use std::sync::{Arc, LazyLock};

static FP_EXPRESSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"{{([^}]*)}}"#).expect("Failed to compile regex"));

pub async fn evaluation<'a, 'b>(
    path: &str,
    values: Vec<&'a dyn MetaValue>,
    config: Arc<Config<'b>>,
) -> Result<String, OperationOutcomeError>
where
    'a: 'b,
{
    let engine = FPEngine::new();

    let mut result = String::new();

    for expression in FP_EXPRESSION_REGEX.captures_iter(path) {
        let full_match = expression.get(0).map(|m| m.as_str()).unwrap_or("");

        let expr = expression.get(1).map(|m| m.as_str()).unwrap_or("");
        if expr.is_empty() {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "FHIRPath expression is empty.".to_string(),
            ));
        }

        let fp_result = engine
            .evaluate_with_config(path, values, config)
            .await
            .map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    format!("FHIRPath evaluation error: {}", e),
                )
            })?;

        let fp_string_result = fp_result
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join(",");

        result.replace(full_match, fp_string_result);
    }

    Ok(result)
}
