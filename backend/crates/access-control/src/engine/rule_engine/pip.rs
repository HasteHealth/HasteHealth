//! Policy Information Point (PIP) module for the access control engine.
//! This module is responsible for retrieving contextual information that can be used during policy evaluation.
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, AccessPolicyV2Attribute, ResourceType},
    terminology::AccessPolicyAttributeOperationTypes,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::ResolvedValue;
use haste_pointer::Pointer;
use haste_reflect::MetaValue;
use std::sync::Arc;

use crate::{context::PolicyContext, engine::rule_engine::expression::evaluate_to_string};

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
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
) -> Result<Option<ResolvedValue>, OperationOutcomeError> {
    let root = pointer.clone();

    match variable_id {
        "user" => Ok(Some(ResolvedValue::Arc(
            policy_context.environment.user.clone() as Arc<dyn MetaValue>,
        ))),
        _ => {
            let access_policy = root.value().ok_or_else(|| {
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

                    let path =
                        evaluate_to_string(policy_context.clone(), pointer, &path_expression)
                            .await?;
                    let reference_chunks = path.split("/").collect::<Vec<_>>();

                    let [resource_type, id] = reference_chunks.as_slice() else {
                        return Err(OperationOutcomeError::fatal(
                            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                            format!(
                                "Attribute operation path '{}' is not a valid resource path for attribute '{}'.",
                                path, variable_id
                            ),
                        ));
                    };

                    let resource_type = ResourceType::try_from(*resource_type).map_err(|_| {
                        OperationOutcomeError::fatal(
                            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                            format!(
                                "Resource type '{}' is not valid for attribute '{}'.",
                                resource_type, variable_id
                            ),
                        )
                    })?;

                    let result = policy_context
                        .client
                        .read(
                            policy_context.client_context.clone(),
                            resource_type,
                            id.to_string(),
                        )
                        .await?;

                    Ok(Some(ResolvedValue::Box(
                        Box::new(result) as Box<dyn MetaValue>
                    )))
                }
                AccessPolicyAttributeOperationTypes::SearchSystem(_) => {
                    let parameter_expression =
                        attribute_operation.params.as_ref().ok_or_else(|| {
                            OperationOutcomeError::fatal(
                                haste_fhir_model::r4::generated::terminology::IssueType::Invalid(
                                    None,
                                ),
                                format!(
                                    "Attribute operation path is not specified for attribute '{}'.",
                                    variable_id
                                ),
                            )
                        })?;

                    let parameters =
                        evaluate_to_string(policy_context.clone(), pointer, &parameter_expression)
                            .await?;

                    let parsed_parameters = ParsedParameters::try_from(parameters.as_str())?;

                    let result = policy_context
                        .client
                        .search_system(policy_context.client_context.clone(), parsed_parameters)
                        .await?;

                    Ok(Some(ResolvedValue::Box(
                        Box::new(result) as Box<dyn MetaValue>
                    )))
                }
                AccessPolicyAttributeOperationTypes::SearchType(_) => {
                    let path_expression = attribute_operation.path.as_ref().ok_or_else(|| {
                        OperationOutcomeError::fatal(
                            haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                            format!(
                                "Attribute operation path is not specified for attribute '{}'.",
                                variable_id
                            ),
                        )
                    })?;

                    let resource_type = evaluate_to_string(
                        policy_context.clone(),
                        pointer.clone(),
                        &path_expression,
                    )
                    .await?;

                    let resource_type =
                        ResourceType::try_from(resource_type.as_str()).map_err(|_| {
                            OperationOutcomeError::fatal(
                                haste_fhir_model::r4::generated::terminology::IssueType::Invalid(
                                    None,
                                ),
                                format!(
                                    "Resource type '{}' is not valid for attribute '{}'.",
                                    resource_type, variable_id
                                ),
                            )
                        })?;

                    let parameter_expression =
                        attribute_operation.params.as_ref().ok_or_else(|| {
                            OperationOutcomeError::fatal(
                                haste_fhir_model::r4::generated::terminology::IssueType::Invalid(
                                    None,
                                ),
                                format!(
                                    "Attribute operation path is not specified for attribute '{}'.",
                                    variable_id
                                ),
                            )
                        })?;

                    let parameters =
                        evaluate_to_string(policy_context.clone(), pointer, &parameter_expression)
                            .await?;

                    let parsed_parameters = ParsedParameters::try_from(parameters.as_str())?;

                    let result = policy_context
                        .client
                        .search_type(
                            policy_context.client_context.clone(),
                            resource_type,
                            parsed_parameters,
                        )
                        .await?;

                    Ok(Some(ResolvedValue::Box(
                        Box::new(result) as Box<dyn MetaValue>
                    )))
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
    }
}
