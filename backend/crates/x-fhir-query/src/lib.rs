use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, FPEngine};
use haste_reflect::MetaValue;
use regex::Regex;
use std::sync::{Arc, LazyLock};

use crate::conversion::stringify_meta_value;

mod conversion;

static FP_EXPRESSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\{\{([^}]*)\}\}"#).expect("Failed to compile regex"));

pub async fn evaluation<'a, 'b>(
    x_fhir_query: &str,
    values: Vec<&'a dyn MetaValue>,
    config: Arc<Config<'b>>,
) -> Result<String, OperationOutcomeError>
where
    'a: 'b,
{
    let engine = FPEngine::new();

    let mut result = x_fhir_query.to_string();

    for expression in FP_EXPRESSION_REGEX.captures_iter(x_fhir_query) {
        let full_match = expression.get(0).map(|m| m.as_str()).unwrap_or("");

        let expr = expression.get(1).map(|m| m.as_str()).unwrap_or("");

        println!("Evaluating FHIRPath expression: '{}'", expr);

        if expr.is_empty() {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "FHIRPath expression is empty.".to_string(),
            ));
        }

        let fp_result = engine
            .evaluate_with_config(expr, values.clone(), config.clone())
            .await
            .map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    format!("FHIRPath evaluation error: {}", e),
                )
            })?;

        let fp_string_result = fp_result
            .iter()
            .map(|v| stringify_meta_value(v))
            .collect::<Result<Vec<String>, OperationOutcomeError>>()?
            .join(",");

        result = result.replace(full_match, &fp_string_result);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::Patient,
        types::{FHIRString, HumanName},
    };
    #[tokio::test]
    async fn test_simple_eval() {
        let patient = Patient {
            id: Some("example".to_string()),

            ..Default::default()
        };
        let result = evaluation(
            "Patient/{{$this.id}}",
            vec![&patient],
            Arc::new(Config {
                variable_resolver: None,
            }),
        )
        .await
        .expect("Evaluation failed");

        assert_eq!(result, "Patient/example");
    }

    #[tokio::test]
    async fn test_multiple() {
        let patient = Patient {
            id: Some("example".to_string()),
            name: Some(vec![Box::new(HumanName {
                family: Some(Box::new(FHIRString {
                    value: Some("Doe".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            })]),
            ..Default::default()
        };
        let result = evaluation(
            "Patient/{{$this.id}}/{{$this.name.family.value}}",
            vec![&patient],
            Arc::new(Config {
                variable_resolver: None,
            }),
        )
        .await
        .expect("Evaluation failed");

        assert_eq!(result, "Patient/example/Doe");
    }
}
