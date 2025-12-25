//! Policy Information Point (PIP) module for the access control engine.
//! This module is responsible for retrieving contextual information that can be used during policy evaluation.
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, AccessPolicyV2Attribute},
    terminology::AccessPolicyAttributeOperationTypes,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Pointer;
use haste_reflect::MetaValue;
use std::sync::Arc;

use crate::{context::PolicyContext, engine::rule_engine::expression::evaluate_expression};

fn find_attribute<'a>(
    access_policy: &'a AccessPolicyV2,
    variable_id: &str,
) -> Option<&'a AccessPolicyV2Attribute> {
    access_policy.attribute.as_ref().and_then(|attributes| {
        attributes
            .iter()
            .find(|a| a.attributeId.value.as_ref().map(|s| s.as_str()) == Some(variable_id))
    })
}

pub async fn pip<
    'a,
    CTX: Sync + Send + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
) -> Result<Option<Box<dyn MetaValue>>, OperationOutcomeError> {
    let access_policy = pointer.clone().value().ok_or_else(|| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            "Pointer root does not contain an AccessPolicyV2 resource.".to_string(),
        )
    })?;

    let Some(attribute) = find_attribute(access_policy, variable_id) else {
        return Ok(None);
    };

    let Some(attribute_operation) = &attribute.operation else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            format!(
                "Attribute operation is not specified for attribute '{}'.",
                variable_id
            ),
        ));
    };

    match attribute_operation.type_.as_ref() {
        AccessPolicyAttributeOperationTypes::Read(_) => {
            let path_expression = attribute_operation.path.as_ref().ok_or_else(|| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                    format!(
                        "Attribute operation path is not specified for attribute '{}'.",
                        variable_id
                    ),
                )
            })?;

            let path = evaluate_expression(policy_context, pointer, &path_expression).await?;

            Ok(None)
        }
        AccessPolicyAttributeOperationTypes::SearchSystem(_) => {
            println!("custom operation");
            Ok(None)
        }
        AccessPolicyAttributeOperationTypes::SearchType(_) => {
            println!("custom operation");
            Ok(None)
        }
        AccessPolicyAttributeOperationTypes::Null(_) => Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
            format!(
                "Attribute operation type is not specified for attribute '{}'.",
                variable_id
            ),
        )),
    }
}
