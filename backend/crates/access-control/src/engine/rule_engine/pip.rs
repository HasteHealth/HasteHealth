//! Policy Information Point (PIP) module for the access control engine.
//! This module is responsible for retrieving contextual information that can be used during policy evaluation.
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_model::r4::generated::{
    resources::{
        AccessPolicyV2, AccessPolicyV2Attribute, AccessPolicyV2AttributeOperation, ResourceType,
    },
    terminology::AccessPolicyAttributeOperationTypes,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::ResolvedValue;
use haste_pointer::TypedPointer;
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
            .find(|a| a.attributeId.value.as_deref() == Some(variable_id))
    })
}

pub async fn pip<
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
) -> Result<Option<ResolvedValue>, OperationOutcomeError> {
    match variable_id {
        "request" => Ok(Some(ResolvedValue::Arc(
            policy_context.environment.request.clone() as Arc<dyn MetaValue>,
        ))),

        "user" => Ok(Some(ResolvedValue::Arc(
            policy_context.environment.user.clone() as Arc<dyn MetaValue>,
        ))),

        _ => evaluate_attribute(policy_context, pointer, variable_id).await,
    }
}

async fn evaluate_attribute<
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: TypedPointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
) -> Result<Option<ResolvedValue>, OperationOutcomeError> {
    let access_policy = pointer.value().ok_or_else(|| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            "Pointer root does not contain an AccessPolicyV2 resource.".to_string(),
        )
    })?;

    let Some(attribute) = find_attribute(access_policy, variable_id) else {
        return Ok(None);
    };

    let Some(attribute_operation) = &attribute.operation else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Attribute operation is not specified for attribute '{variable_id}'."),
        ));
    };

    match &attribute_operation.type_ {
        attribute_type if attribute_type == &AccessPolicyAttributeOperationTypes::read() => {
            evaluate_read(policy_context, &pointer, variable_id, attribute_operation).await
        }

        attribute_type
            if attribute_type == &AccessPolicyAttributeOperationTypes::search_system() =>
        {
            evaluate_search_system(policy_context, &pointer, variable_id, attribute_operation).await
        }

        attribute_type if attribute_type == &AccessPolicyAttributeOperationTypes::search_type() => {
            evaluate_search_type(policy_context, &pointer, variable_id, attribute_operation).await
        }

        attribute_type if attribute_type == &AccessPolicyAttributeOperationTypes::null() => {
            Err(OperationOutcomeError::fatal(
                haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                format!("Attribute operation type is not specified for attribute '{variable_id}'."),
            ))
        }
        _ => Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!(
                "Attribute operation type '{:?}' is not supported for attribute '{}'.",
                attribute_operation.type_, variable_id
            ),
        )),
    }
}

async fn evaluate_read<
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: &TypedPointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
    operation: &AccessPolicyV2AttributeOperation,
) -> Result<Option<ResolvedValue>, OperationOutcomeError> {
    let path_expression = operation.path.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Attribute operation path is not specified for attribute '{variable_id}'."),
        )
    })?;

    let path = evaluate_to_string(policy_context.clone(), pointer.clone(), path_expression).await?;

    let reference_chunks = path.split('/').collect::<Vec<_>>();

    let [resource_type, id] = reference_chunks.as_slice() else {
        return Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!(
                "Attribute operation path '{path}' is not a valid resource path for attribute '{variable_id}'."
            ),
        ));
    };

    let resource_type = ResourceType::try_from(*resource_type).map_err(|_| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Resource type '{resource_type}' is not valid for attribute '{variable_id}'."),
        )
    })?;

    let result = policy_context
        .client
        .read(
            policy_context.client_context.clone(),
            resource_type,
            (*id).to_string(),
        )
        .await?;

    Ok(Some(ResolvedValue::Box(
        Box::new(result) as Box<dyn MetaValue>
    )))
}

async fn evaluate_search_system<
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: &TypedPointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
    operation: &AccessPolicyV2AttributeOperation,
) -> Result<Option<ResolvedValue>, OperationOutcomeError> {
    let parameter_expression = operation.params.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Attribute operation params are not specified for attribute '{variable_id}'."),
        )
    })?;

    let parameters = evaluate_to_string(
        policy_context.clone(),
        pointer.clone(),
        parameter_expression,
    )
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

async fn evaluate_search_type<
    CTX: Sync + Send + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    pointer: &TypedPointer<AccessPolicyV2, AccessPolicyV2>,
    variable_id: &str,
    operation: &AccessPolicyV2AttributeOperation,
) -> Result<Option<ResolvedValue>, OperationOutcomeError> {
    let path_expression = operation.path.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Attribute operation path is not specified for attribute '{variable_id}'."),
        )
    })?;

    let resource_type =
        evaluate_to_string(policy_context.clone(), pointer.clone(), path_expression).await?;

    let resource_type = ResourceType::try_from(resource_type.as_str()).map_err(|_| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Resource type '{resource_type}' is not valid for attribute '{variable_id}'."),
        )
    })?;

    let parameter_expression = operation.params.as_ref().ok_or_else(|| {
        OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            format!("Attribute operation params are not specified for attribute '{variable_id}'."),
        )
    })?;

    let parameters = evaluate_to_string(
        policy_context.clone(),
        pointer.clone(),
        parameter_expression,
    )
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
