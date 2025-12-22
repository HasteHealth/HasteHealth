use crate::context::PolicyContext;
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::IssueType, types::Expression,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Context, ExternalConstantResolver, FHIRPathError};
use std::sync::Arc;

pub fn create_config() -> haste_fhirpath::Config<'static> {
    haste_fhirpath::Config {
        variable_resolver: Some(ExternalConstantResolver::Function(Box::new(
            |_variable_id| {
                Box::pin(async move {
                    todo!();
                })
            },
        ))),
    }
}

pub async fn evaluate_expression<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: Arc<PolicyContext<CTX, Client>>,
    policy: &'a AccessPolicyV2,
    expression: &Expression,
) -> Result<Context<'a>, OperationOutcomeError> {
    match (
        expression
            .language
            .as_ref()
            .value
            .as_ref()
            .map(|s| s.as_str()),
        expression
            .expression
            .as_ref()
            .and_then(|s| s.value.as_ref()),
    ) {
        (Some("text/fhirpath"), Some(expr)) => {
            let result = context
                .fp_engine
                .evaluate_with_config(expr, vec![policy], Arc::new(create_config()))
                .await
                .map_err(|e: FHIRPathError| {
                    OperationOutcomeError::fatal(
                        IssueType::NotSupported(None),
                        format!("FHIRPath evaluation error: {}", e),
                    )
                })?;

            Ok(result)
        }
        _ => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "Expression language not supported.".to_string(),
        )),
    }
}
