use crate::{context::PolicyContext, engine::rule_engine::pip::pip};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::IssueType, types::Expression,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Context, ExternalConstantResolver, FHIRPathError};
use haste_pointer::Pointer;
use std::sync::Arc;

pub fn create_config<
    'a,
    CTX: Sync + Send + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
) -> haste_fhirpath::Config<'a> {
    haste_fhirpath::Config {
        variable_resolver: Some(ExternalConstantResolver::Function(Box::new(
            move |variable_id: String| {
                let pointer = pointer.clone();
                let context = context.clone();
                Box::pin(async move {
                    let Ok(access_policy_v2) = pointer.value().ok_or_else(|| {
                        OperationOutcomeError::fatal(
                            IssueType::Invalid(None),
                            "Pointer root does not contain an AccessPolicyV2 resource.".to_string(),
                        )
                    }) else {
                        return None;
                    };

                    let Some(_result) = pip(context, access_policy_v2, &variable_id).await.ok()
                    else {
                        return None;
                    };

                    None
                })
            },
        ))),
    }
}

pub async fn evaluate_expression<
    'a,
    CTX: Sync + Send + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
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
                .evaluate_with_config(
                    expr,
                    vec![],
                    Arc::new(create_config(context.clone(), pointer)),
                )
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
