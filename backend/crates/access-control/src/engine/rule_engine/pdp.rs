use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, AccessPolicyV2Rule, AccessPolicyV2RuleTarget},
    terminology::{AccessPolicyRuleEffect, AccessPolicyv2CombineBehavior, IssueType},
    types::FHIRBoolean,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_pointer::TypedPointer;
use std::sync::Arc;

use crate::{
    context::{PermissionLevel, PermissionLevelError, PolicyContext},
    engine::rule_engine::expression::evaluate_expression,
    utilities::{get_max, get_min},
};

#[derive(Debug, OperationOutcomeError)]
pub enum PDPError {
    #[error(code = "exception", diagnostic = "Pointer at '{arg0}' failed.")]
    PointerError(String),
    #[error(code = "invalid", diagnostic = "{arg0:?}")]
    InvalidPermissionLevel(PermissionLevelError),
}

type PolicyResult<T, Context> = (T, Context);

async fn should_evaluate_rule<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Option<TypedPointer<AccessPolicyV2, AccessPolicyV2RuleTarget>>,
) -> Result<PolicyResult<bool, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let Some(pointer) = pointer else {
        // If no target is specified, always evaluate the rule.
        return Ok((true, context));
    };

    let Some(target) = pointer.value() else {
        // If no target is specified, always evaluate the rule.
        return Ok((true, context));
    };

    let root = pointer.root();

    let result = evaluate_expression(context.clone(), root, target.expression.as_ref()).await?;
    let values = result.iter().collect::<Vec<_>>();

    if values.len() != 1 {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!(
                "Target expression at '{}' did not evaluate to a single value.",
                pointer.path()
            ),
        ));
    }

    let Some(should_evaluate_the_rule) = values[0]
        .as_any()
        .downcast_ref::<FHIRBoolean>()
        .and_then(|b| b.value)
    else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!(
                "Target expression did not evaluate to a boolean value it resolved to '{}'",
                values[0].fhir_type()
            ),
        ));
    };

    Ok((should_evaluate_the_rule, context))
}

fn coalesce_boolean(
    values: &Vec<&dyn haste_reflect::MetaValue>,
) -> Result<bool, OperationOutcomeError> {
    if values.len() != 1 {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            "Condition expression did not evaluate to a single value.".to_string(),
        ));
    }

    let Some(value) = values.first() else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            "Condition expression did not evaluate to a value.".to_string(),
        ));
    };

    match value.fhir_type() {
        "boolean" => value
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .and_then(|b| b.value)
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                    "Condition expression evaluated to a FHIRBoolean with no value.".to_string(),
                )
            }),
        "http://hl7.org/fhirpath/System.Boolean" => value
            .as_any()
            .downcast_ref::<bool>()
            .copied()
            .ok_or_else(|| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                    "Condition expression evaluated to a System.Boolean with no value.".to_string(),
                )
            }),
        _ => Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            "Condition expression did not evaluate to a boolean value.".to_string(),
        )),
    }
}

async fn evaluate_condition<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;
    let condition = rule.condition.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
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
        .unwrap_or(AccessPolicyRuleEffect::permit());

    if should_permit {
        match effect {
            effect
                if effect == AccessPolicyRuleEffect::null()
                    || effect == AccessPolicyRuleEffect::permit() =>
            {
                Ok((PermissionLevel::Allow, policy_context.clone()))
            }
            _effect => Ok((PermissionLevel::Deny, policy_context.clone())),
        }
    } else {
        match effect {
            effect if effect == AccessPolicyRuleEffect::deny() => {
                Ok((PermissionLevel::Allow, policy_context.clone()))
            }
            _effect => Ok((PermissionLevel::Deny, policy_context.clone())),
        }
    }
}

async fn evaluate_access_policy_rule<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;

    let (should_evaluate, policy_context) = should_evaluate_rule(
        policy_context,
        rule_pointer
            .descend::<AccessPolicyV2RuleTarget>(&haste_pointer::Key::Field("target".to_string())),
    )
    .await?;

    if !should_evaluate {
        return Ok((PermissionLevel::Undetermined, policy_context));
    }

    match rule.combineBehavior.as_ref() {
        combine_behavior if combine_behavior == Some(&AccessPolicyv2CombineBehavior::any()) => {
            evaluate_any_rules(policy_context, rule_pointer).await
        }

        combine_behavior if combine_behavior == Some(&AccessPolicyv2CombineBehavior::all_of()) => {
            evaluate_all_of_rules(policy_context, rule_pointer).await
        }

        combine_behavior
            if combine_behavior == Some(&AccessPolicyv2CombineBehavior::null())
                || combine_behavior.is_none() =>
        {
            evaluate_leaf_rule(policy_context, rule_pointer).await
        }
        _ => Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Unsupported combineBehavior value.".to_string(),
        )),
    }
}

async fn evaluate_any_rules<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    mut policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;

    if rule.condition.is_some() {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Condition is not supported when combineBehavior is 'any'.".to_string(),
        ));
    }

    let mut result = PermissionLevel::Deny;

    for (index, _) in rule.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let nested_rule_pointer = get_nested_rule_pointer(&rule_pointer, index)?;

        let (permission, next_context) = Box::pin(evaluate_access_policy_rule(
            policy_context.clone(),
            nested_rule_pointer,
        ))
        .await?;

        result = get_max(&result, &permission)?;
        policy_context = next_context;
    }

    Ok((result, policy_context))
}

async fn evaluate_all_of_rules<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    mut policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;

    if rule.condition.is_some() {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Condition is not supported when combineBehavior is 'allOf'.".to_string(),
        ));
    }

    let mut result = PermissionLevel::Allow;

    for (index, _) in rule.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let nested_rule_pointer = get_nested_rule_pointer(&rule_pointer, index)?;

        let (permission, next_context) = Box::pin(evaluate_access_policy_rule(
            policy_context.clone(),
            nested_rule_pointer,
        ))
        .await?;

        if permission == PermissionLevel::Deny {
            return Ok((PermissionLevel::Deny, next_context));
        }

        result = get_min(&result, &permission)?;
        policy_context = next_context;
    }

    Ok((result, policy_context))
}

async fn evaluate_leaf_rule<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    rule_pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<PolicyResult<PermissionLevel, Arc<PolicyContext<CTX, Client>>>, OperationOutcomeError> {
    let rule = rule_pointer
        .value()
        .ok_or(PDPError::PointerError(rule_pointer.path().to_string()))?;

    if rule.rule.is_some() {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Nested rules are not supported when combineBehavior is 'null' or unspecified."
                .to_string(),
        ));
    }

    evaluate_condition(policy_context, rule_pointer).await
}

fn get_nested_rule_pointer(
    rule_pointer: &TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>,
    index: usize,
) -> Result<TypedPointer<AccessPolicyV2, AccessPolicyV2Rule>, PDPError> {
    rule_pointer
        .descend::<Vec<AccessPolicyV2Rule>>(&haste_pointer::Key::Field("rule".to_string()))
        .and_then(|p| p.descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(index)))
        .ok_or_else(|| PDPError::PointerError(format!("{}/rule/{}", rule_pointer.path(), index)))
}

pub async fn evaluate<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    mut policy_context: Arc<PolicyContext<CTX, Client>>,
    policy: Arc<AccessPolicyV2>,
) -> Result<PermissionLevel, OperationOutcomeError> {
    let pointer = TypedPointer::<AccessPolicyV2, AccessPolicyV2>::new(policy.clone());

    let rules_collection_pointer = pointer
        .descend::<Vec<AccessPolicyV2Rule>>(&haste_pointer::Key::Field("rule".to_string()))
        .ok_or_else(|| PDPError::PointerError("rule".to_string()))?;

    let mut result = PermissionLevel::Deny;

    for (index, _) in policy.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let rule_pointer = rules_collection_pointer
            .descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(index))
            .ok_or_else(|| {
                PDPError::PointerError(format!("{}/{}", rules_collection_pointer.path(), index))
            })?;

        match evaluate_access_policy_rule(policy_context.clone(), rule_pointer).await? {
            (PermissionLevel::Deny, _) => return Ok(PermissionLevel::Deny),
            (permission_level, context) => {
                policy_context = context;
                result = get_max(&result, &permission_level)?;
            }
        }
    }

    Ok(result)
}
