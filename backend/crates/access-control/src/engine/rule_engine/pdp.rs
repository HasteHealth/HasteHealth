use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, AccessPolicyV2Rule, AccessPolicyV2RuleTarget},
    terminology::{AccessPolicyRuleEffect, AccessPolicyv2CombineBehavior, IssueType},
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

fn get_min(p1: &PermissionLevel, p2: &PermissionLevel) -> Result<PermissionLevel, PDPError> {
    let min = std::cmp::min(i8::from(p1), i8::from(p2));

    PermissionLevel::try_from(min).map_err(PDPError::InvalidPermissionLevel)
}

async fn should_evaluate_rule<
    'a,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2RuleTarget>,
) -> Result<PolicyResult<bool, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let Some(target) = pointer.value() else {
        // If no target is specified, always evaluate the rule.
        return Ok((true, context));
    };

    let root = pointer.root();

    let result = evaluate_expression(context.clone(), root, target.expression.as_ref()).await?;

    let values = result.iter().collect::<Vec<_>>();

    if values.len() != 1 {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            "Target expression did not evaluate to a single boolean value.".to_string(),
        ));
    }

    let Some(should_evaluate_the_rule) = values[0]
        .as_any()
        .downcast_ref::<FHIRBoolean>()
        .and_then(|b| b.value)
    else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            "Target expression did not evaluate to a boolean value.".to_string(),
        ));
    };

    Ok((should_evaluate_the_rule, context))
}

fn coalesce_boolean(
    values: &Vec<&dyn haste_reflect::MetaValue>,
) -> Result<bool, OperationOutcomeError> {
    if values.len() != 1 {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            "Condition expression did not evaluate to a single value.".to_string(),
        ));
    }

    let Some(value) = values.get(0) else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            "Condition expression did not evaluate to a value.".to_string(),
        ));
    };

    match value.typename() {
        "FHIRBoolean" => value
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .and_then(|b| b.value)
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                    "Condition expression evaluated to a FHIRBoolean with no value.".to_string(),
                )
            }),
        "http://hl7.org/fhirpath/System.Boolean" => value
            .as_any()
            .downcast_ref::<bool>()
            .copied()
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                    "Condition expression evaluated to a System.Boolean with no value.".to_string(),
                )
            }),
        _ => {
            return Err(OperationOutcomeError::fatal(
                haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                "Condition expression did not evaluate to a boolean value.".to_string(),
            ));
        }
    }
}

async fn evaluate_condition<
    'a,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: Pointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;
    let condition = rule.condition.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            "Condition is not specified for the rule.".to_string(),
        )
    })?;

    let condition_result = evaluate_expression(
        policy_context.clone(),
        rule_pointer.root(),
        condition.expression.as_ref(),
    )
    .await?;

    let should_permit = coalesce_boolean(&condition_result.iter().collect())?;

    let effect = rule
        .effect
        .clone()
        .unwrap_or(Box::new(AccessPolicyRuleEffect::Permit(None)));

    if should_permit {
        match effect.as_ref() {
            AccessPolicyRuleEffect::Null(_) | AccessPolicyRuleEffect::Permit(_) => {
                Ok((PermissionLevel::Allow, policy_context.clone()))
            }
            AccessPolicyRuleEffect::Deny(_) => Ok((PermissionLevel::Deny, policy_context.clone())),
        }
    } else {
        match effect.as_ref() {
            AccessPolicyRuleEffect::Null(_) | AccessPolicyRuleEffect::Permit(_) => {
                Ok((PermissionLevel::Deny, policy_context.clone()))
            }
            AccessPolicyRuleEffect::Deny(_) => Ok((PermissionLevel::Allow, policy_context.clone())),
        }
    }
}

async fn evaluate_access_policy_rule<
    'a,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: Pointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;

    let (should_evaluate, mut policy_context) = should_evaluate_rule(
        policy_context,
        rule_pointer
            .descend::<AccessPolicyV2RuleTarget>(&haste_pointer::Key::Field("target".to_string()))
            .ok_or_else(|| PDPError::PointerError(format!("{}/target", rule_pointer.path())))?,
    )
    .await?;

    if !should_evaluate {
        return Ok((PermissionLevel::Undetermined, policy_context));
    }

    match rule.combineBehavior.as_ref().map(|s| s.as_ref()) {
        Some(AccessPolicyv2CombineBehavior::Any(_)) => {
            let mut result = PermissionLevel::Undetermined;
            if rule.condition.is_some() {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "Condition is not supported when combineBehavior is 'any'.".to_string(),
                ));
            }

            for (nested_index, _) in rule.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
                let nested_rule_pointer = rule_pointer
                    .descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Field("rule".to_string()))
                    .and_then(|p| {
                        p.descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(nested_index))
                    })
                    .ok_or_else(|| {
                        PDPError::PointerError(format!(
                            "{}/rule/{}",
                            rule_pointer.path(),
                            nested_index
                        ))
                    })?;

                let context = policy_context.clone();

                let rule_result: Result<
                    (PermissionLevel, Arc<PolicyContext<CTX, Client>>),
                    OperationOutcomeError,
                > = Box::pin(async move {
                    let nested_rule_result =
                        evaluate_access_policy_rule(context.clone(), nested_rule_pointer).await?;

                    Ok(nested_rule_result)
                })
                .await;

                let (rule_result, next_context) = rule_result?;

                // Any logic means if any rule grants access, access is granted. So we can just take the max permission allowed.
                result = get_min(&result, &rule_result)?;

                policy_context = next_context;
            }

            Ok((result, policy_context))
        }
        Some(AccessPolicyv2CombineBehavior::AllOf(_)) => {
            // Set as allowed because doing min logic below.
            let mut result = PermissionLevel::Allow;
            if rule.condition.is_some() {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "Condition is not supported when combineBehavior is 'any'.".to_string(),
                ));
            }

            for (nested_index, _) in rule.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
                let nested_rule_pointer = rule_pointer
                    .descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Field("rule".to_string()))
                    .and_then(|p| {
                        p.descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(nested_index))
                    })
                    .ok_or_else(|| {
                        PDPError::PointerError(format!(
                            "{}/rule/{}",
                            rule_pointer.path(),
                            nested_index
                        ))
                    })?;
                let context = policy_context.clone();

                let rule_result: Result<
                    (PermissionLevel, Arc<PolicyContext<CTX, Client>>),
                    OperationOutcomeError,
                > = Box::pin(async move {
                    let nested_rule_result =
                        evaluate_access_policy_rule(context.clone(), nested_rule_pointer).await?;

                    Ok(nested_rule_result)
                })
                .await;

                let (rule_result, next_context) = rule_result?;

                // And logic means minimum permission level is taken.
                result = get_min(&result, &rule_result)?;

                policy_context = next_context;
            }

            Ok((result, policy_context))
        }
        Some(&AccessPolicyv2CombineBehavior::Null(_)) | None => {
            if rule.rule.is_some() {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "Nested rules are not supported when combineBehavior is 'null' or unspecified."
                        .to_string(),
                ));
            }

            let result = evaluate_condition(policy_context, rule_pointer).await?;

            Ok(result)
        }
    }
}

pub async fn evaluate<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    mut policy_context: Arc<PolicyContext<CTX, Client>>,
    policy: Arc<AccessPolicyV2>,
) -> Result<PermissionLevel, OperationOutcomeError> {
    let pointer = Pointer::<AccessPolicyV2, AccessPolicyV2>::new(policy.clone());
    let rules_pointer = pointer
        .descend::<Option<Vec<AccessPolicyV2Rule>>>(&haste_pointer::Key::Field("rule".to_string()))
        .ok_or_else(|| PDPError::PointerError("rule".to_string()))?;

    let mut result = PermissionLevel::Deny;

    for (index, _) in policy.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let rule_pointer = rules_pointer
            .descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(index))
            .ok_or_else(|| PDPError::PointerError(format!("{}/{}", rules_pointer.path(), index)))?;

        match evaluate_access_policy_rule(policy_context.clone(), rule_pointer).await? {
            (PermissionLevel::Deny, _) => return Ok(PermissionLevel::Deny),
            (permission_level, context) => {
                // Continue evaluating other rules
                policy_context = context;

                result = get_max(&result, &permission_level)?;
            }
        }
    }

    Ok(result)
}
