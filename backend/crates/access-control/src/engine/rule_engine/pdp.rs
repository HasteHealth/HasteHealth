use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, AccessPolicyV2Rule, AccessPolicyV2RuleTarget},
    types::FHIRBoolean,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_pointer::Pointer;
use std::sync::Arc;

use crate::{
    context::{PermissionLevel, PermissionLevelError, PolicyContext},
    engine::rule_engine::expression::evaluate_expression,
};

#[derive(Debug, OperationOutcomeError)]
pub enum PDPError {
    #[error(code = "exception", diagnostic = "Pointer at '{arg0}' failed.")]
    PointerError(String),
    #[error(code = "invalid", diagnostic = "{arg0:?}")]
    InvalidPermissionLevel(PermissionLevelError),
}

type PolicyResult<T, Context> = (T, Context);

fn get_max(p1: &PermissionLevel, p2: &PermissionLevel) -> Result<PermissionLevel, PDPError> {
    let max = std::cmp::max(i8::from(p1), i8::from(p2));

    PermissionLevel::try_from(max).map_err(PDPError::InvalidPermissionLevel)
}

#[allow(unused)]
fn resolve_variable<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    _context: Arc<PolicyContext<CTX, Client>>,
    _pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}

async fn should_evaluate_rule<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<'a, AccessPolicyV2, AccessPolicyV2RuleTarget>,
) -> Result<PolicyResult<bool, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let Some(target) = pointer.value() else {
        // If no target is specified, always evaluate the rule.
        return Ok((true, context));
    };

    let result = evaluate_expression(
        context.clone(),
        pointer.root().value().unwrap(),
        target.expression.as_ref(),
    )
    .await?;

    let values = result.iter().collect::<Vec<_>>();

    if values.len() != 1 {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            "Target expression did not evaluate to a single boolean value.".to_string(),
        ));
    }

    let should_evaluate_the_rule = values[0]
        .as_any()
        .downcast_ref::<FHIRBoolean>()
        .and_then(|b| b.value);

    Ok((should_evaluate_the_rule.unwrap_or(false), context))
}

async fn evaluate_access_policy_rule<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: Pointer<'a, AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<(Arc<PolicyContext<CTX, Client>>, PermissionLevel), OperationOutcomeError> {
    let _rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;

    let (should_evaluate, policy_context) = should_evaluate_rule(
        policy_context,
        rule_pointer
            .descend::<AccessPolicyV2RuleTarget>(&haste_pointer::Key::Field("target".to_string()))
            .ok_or_else(|| PDPError::PointerError(format!("{}/target", rule_pointer.path())))?,
    )
    .await?;

    if !should_evaluate {
        return Ok((policy_context, PermissionLevel::Undetermined));
    }

    Ok((policy_context, PermissionLevel::Deny))
}

#[allow(unused)]
pub async fn evaluate<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    mut policy_context: Arc<PolicyContext<CTX, Client>>,
    policy: &AccessPolicyV2,
) -> Result<PermissionLevel, OperationOutcomeError> {
    let pointer = Pointer::<AccessPolicyV2, AccessPolicyV2>::new(policy);
    let rules_pointer = pointer
        .descend::<Option<Vec<AccessPolicyV2Rule>>>(&haste_pointer::Key::Field("rule".to_string()))
        .ok_or_else(|| PDPError::PointerError("rule".to_string()))?;

    let mut result = PermissionLevel::Deny;

    for (index, _) in policy.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let rule_pointer = rules_pointer
            .descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(index))
            .ok_or_else(|| PDPError::PointerError(format!("{}/{}", rules_pointer.path(), index)))?;

        match evaluate_access_policy_rule(policy_context.clone(), rule_pointer).await? {
            (_, PermissionLevel::Deny) => return Ok(PermissionLevel::Deny),
            (context, permission_level) => {
                // Continue evaluating other rules
                policy_context = context;

                result = get_max(&result, &permission_level)?;
            }
        }
    }

    Ok(result)
}
