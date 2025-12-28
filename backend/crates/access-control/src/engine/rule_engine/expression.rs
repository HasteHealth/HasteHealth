use crate::{context::PolicyContext, engine::rule_engine::pip::pip};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::IssueType, types::Expression,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Context, ExternalConstantResolver, FHIRPathError};
use haste_pointer::Pointer;
use haste_reflect::MetaValue;
use std::sync::Arc;

pub fn create_config<
    'a,
    CTX: Sync + Send + Clone + 'static,
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
                    if let Some(result) = pip(context, pointer, &variable_id).await.ok() {
                        result
                    } else {
                        None
                    }
                })
            },
        ))),
    }
}

pub enum ExpressionResult<'a> {
    FHIRPath(Context<'a>),
    XFHIRQuery(Vec<String>),
}

impl<'a> ExpressionResult<'a> {
    pub fn iter(&'a self) -> Box<dyn Iterator<Item = &'a dyn MetaValue> + 'a> {
        match self {
            ExpressionResult::FHIRPath(ctx) => ctx.iter(),
            ExpressionResult::XFHIRQuery(res) => Box::new(res.iter().map(|v| v as &dyn MetaValue)),
        }
    }
}

pub async fn evaluate_expression<
    'a,
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
    expression: &Expression,
) -> Result<ExpressionResult<'a>, OperationOutcomeError> {
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

            Ok(ExpressionResult::FHIRPath(result))
        }
        (Some("application/x-fhir-query"), Some(expr)) => {
            let result = haste_x_fhir_query::evaluation(
                expr,
                vec![],
                Arc::new(create_config(context.clone(), pointer)),
            )
            .await?;

            Ok(ExpressionResult::XFHIRQuery(vec![result]))
        }
        _ => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "Expression language not supported.".to_string(),
        )),
    }
}

pub async fn evaluate_to_string<
    'a,
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
    expression: &Expression,
) -> Result<String, OperationOutcomeError> {
    let result = evaluate_expression(context, pointer, expression).await?;
    let result = result.iter().collect::<Vec<_>>();

    if result.len() != 1 {
        return Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            "Expression did not evaluate to a single value.".to_string(),
        ));
    }

    let string = haste_x_fhir_query::conversion::stringify_meta_value(result[0])?;
    Ok(string)
}
