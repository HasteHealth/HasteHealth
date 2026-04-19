use std::sync::Arc;

use haste_fhir_client::url::{ParsedParameter, ParsedParameters};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, SearchParameter, Subscription},
    terminology::IssueType,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhir_search::{
    SearchParameterResolve,
    indexing_conversion::{self, InsertableIndex},
};
use haste_jwt::{ProjectId, TenantId};

pub mod traits;

#[derive(OperationOutcomeError, Debug)]
pub enum SubscriptionFilterError {
    #[fatal(
        code = "exception",
        diagnostic = "Failed to evaluate fhirpath expression."
    )]
    FHIRPathError(#[from] haste_fhirpath::FHIRPathError),
}

#[allow(dead_code)]
pub struct SubscriptionParameter {
    search_parameter: Arc<SearchParameter>,
    fp_extract_expression: String,
    value: Vec<String>,
    modifier: Option<String>,
}

pub enum SubscriptionTrigger {
    // Based around simple Subscription.criteria.
    QueryFilter {
        resource_type: ResourceType,
        parameters: Vec<SubscriptionParameter>,
    },
    // This could come from a subscriptiontopic which
    // allows arbitrary FHIRPath expressions, or from more complex criteria in the future.
    FHIRPathFilter {
        expression: String,
    },
}

/// In memory representation of a subscription filter.
/// This is what we will use to evaluate whether a given subscription matches an incoming event.
#[allow(dead_code)]
pub struct MemorySubscriptionFilter {
    fp_engine: haste_fhirpath::FPEngine,
    triggers: Vec<SubscriptionTrigger>,
}

impl MemorySubscriptionFilter {
    pub async fn new<Resolver: SearchParameterResolve>(
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resolver: Arc<Resolver>,
        value: Subscription,
    ) -> Result<Self, OperationOutcomeError> {
        if let Some(criteria) = value.criteria.value {
            let criteria_pieces = criteria.split('?').collect::<Vec<_>>();
            let [path, parameters] = criteria_pieces.as_slice() else {
                return Err(OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Invalid subscription criteria format".to_string(),
                ));
            };

            let resource_type = ResourceType::try_from(*path).map_err(|_| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Invalid resource type".to_string(),
                )
            })?;

            let parsed_parameters = ParsedParameters::try_from(*parameters)?;
            let mut subscription_parsed_parameters = vec![];

            for parameter in parsed_parameters.owned_parameters().into_iter() {
                match parameter {
                    ParsedParameter::Resource(resource_param) => {
                        let Some(search_parameter) = resolver
                            .by_name(
                                tenant_id,
                                project_id,
                                Some(&resource_type),
                                &resource_param.name,
                            )
                            .await?
                        else {
                            return Err(OperationOutcomeError::error(
                                IssueType::Exception(None),
                                format!(
                                    "Invalid search parameter in subscription criteria: {}",
                                    resource_param.name
                                ),
                            ));
                        };

                        if resource_param.chains.is_some() {
                            return Err(OperationOutcomeError::error(
                                IssueType::Exception(None),
                                format!(
                                    "Chained parameters are not supported in subscription criteria: {}",
                                    resource_param.name
                                ),
                            ));
                        }

                        let Some(fp_expression) = search_parameter
                            .expression
                            .as_ref()
                            .and_then(|expr| expr.value.as_ref())
                        else {
                            return Err(OperationOutcomeError::error(
                                IssueType::Exception(None),
                                format!(
                                    "Search parameter does not have an expression: {}",
                                    resource_param.name
                                ),
                            ));
                        };

                        subscription_parsed_parameters.push(SubscriptionParameter {
                            search_parameter: search_parameter.clone(),
                            fp_extract_expression: fp_expression.clone(),
                            value: resource_param.value,
                            modifier: resource_param.modifier,
                        });
                    }
                    ParsedParameter::Result(result_param) => {
                        return Err(OperationOutcomeError::error(
                            IssueType::Exception(None),
                            format!(
                                "Unsupported parameter in subscription criteria: {}",
                                result_param.name
                            ),
                        ));
                    }
                }
            }

            Ok(MemorySubscriptionFilter {
                fp_engine: haste_fhirpath::FPEngine::new(),
                triggers: vec![SubscriptionTrigger::QueryFilter {
                    resource_type,
                    parameters: subscription_parsed_parameters,
                }],
            })
        } else {
            Err(OperationOutcomeError::error(
                IssueType::Exception(None),
                "SubscriptionFilter conversion not implemented".to_string(),
            ))
        }
    }
}

async fn fits_subscription_parameter(
    fp_engine: &haste_fhirpath::FPEngine,
    subscription_parameter: &SubscriptionParameter,
    resource: &Resource,
) -> Result<bool, OperationOutcomeError> {
    let result = fp_engine
        .evaluate(
            &subscription_parameter.fp_extract_expression,
            vec![resource],
        )
        .await
        .map_err(SubscriptionFilterError::from)?;

    let conversions = indexing_conversion::to_insertable_index(
        &subscription_parameter.search_parameter.as_ref(),
        result.iter().collect::<Vec<_>>(),
    )?;

    match conversions {
        InsertableIndex::String(resource_values) => {
            Ok(resource_values.iter().any(|resource_value| {
                subscription_parameter
                    .value
                    .iter()
                    .any(|v| resource_value.to_lowercase().starts_with(&v.to_lowercase()))
            }))
        }
        InsertableIndex::Number(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Number search parameters are not supported in subscription criteria".to_string(),
        ))?,
        InsertableIndex::URI(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "URI search parameters are not supported in subscription criteria".to_string(),
        ))?,
        InsertableIndex::Token(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Token search parameters are not supported in subscription criteria".to_string(),
        ))?,
        InsertableIndex::Date(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Date search parameters are not supported in subscription criteria".to_string(),
        ))?,

        InsertableIndex::Reference(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Reference search parameters are not supported in subscription criteria".to_string(),
        ))?,
        InsertableIndex::Quantity(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Quantity search parameters are not supported in subscription criteria".to_string(),
        ))?,

        InsertableIndex::Composite(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Composite search parameters are not supported in subscription criteria".to_string(),
        ))?,
        InsertableIndex::Special(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Special search parameters are not supported in subscription criteria".to_string(),
        ))?,
        InsertableIndex::Meta(_) => Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Meta search parameters are not supported in subscription criteria".to_string(),
        ))?,
    }
}

impl traits::SubscriptionFilter for MemorySubscriptionFilter {
    async fn matches(&self, resource: &Resource) -> Result<bool, OperationOutcomeError> {
        let resource_resource_type = resource.resource_type();

        for trigger in self.triggers.iter() {
            match trigger {
                SubscriptionTrigger::QueryFilter {
                    resource_type,
                    parameters,
                } => {
                    if *resource_type != resource_resource_type {
                        return Ok(false);
                    }

                    for sub_parameter in parameters {
                        let fits_criteria =
                            fits_subscription_parameter(&self.fp_engine, sub_parameter, resource)
                                .await?;
                        if !fits_criteria {
                            return Ok(false);
                        }
                    }

                    return Ok(true);
                }
                SubscriptionTrigger::FHIRPathFilter { .. } => {
                    return Err(OperationOutcomeError::error(
                        IssueType::Exception(None),
                        "FHIRPathFilter triggers are not yet supported".to_string(),
                    ))?;
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use haste_fhir_model::r4::generated::{
        resources::Patient,
        types::{FHIRString, HumanName},
    };
    use haste_fhir_search::memory::R4_SEARCH_PARAMETERS_INDEX;

    use crate::traits::SubscriptionFilter;

    use super::*;

    #[tokio::test]
    async fn quick_test_derive() {
        let subscription = Subscription {
            criteria: Box::new(FHIRString {
                value: Some("Patient?name=Smith".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let resolver = R4_SEARCH_PARAMETERS_INDEX.clone();

        let sub_filter = MemorySubscriptionFilter::new(
            &TenantId::System,
            &ProjectId::System,
            resolver,
            subscription,
        )
        .await
        .unwrap();

        assert_eq!(sub_filter.triggers.len(), 1);

        match &sub_filter.triggers[0] {
            SubscriptionTrigger::QueryFilter {
                resource_type,
                parameters,
            } => {
                assert_eq!(resource_type, &ResourceType::Patient);
                assert_eq!(parameters[0].fp_extract_expression, "Patient.name");
                assert_eq!(parameters[0].value, vec!["Smith".to_string()]);
            }
            _ => panic!("Expected QueryFilter trigger"),
        };
    }

    #[tokio::test]
    async fn modifier_check() {
        let subscription = Subscription {
            criteria: Box::new(FHIRString {
                value: Some("Observation?category:missing=true".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let resolver = R4_SEARCH_PARAMETERS_INDEX.clone();
        let sub_filter = MemorySubscriptionFilter::new(
            &TenantId::System,
            &ProjectId::System,
            resolver,
            subscription,
        )
        .await
        .unwrap();

        assert_eq!(sub_filter.triggers.len(), 1);

        match &sub_filter.triggers[0] {
            SubscriptionTrigger::QueryFilter {
                resource_type,
                parameters,
            } => {
                assert_eq!(resource_type, &ResourceType::Observation);
                assert_eq!(parameters[0].fp_extract_expression, "Observation.category");
                assert_eq!(parameters[0].value, vec!["true".to_string()]);
                assert_eq!(parameters[0].modifier, Some("missing".to_string()));
            }
            _ => panic!("Expected QueryFilter trigger"),
        };
    }

    #[tokio::test]
    async fn test_run_fhirpath() {
        let resolver = R4_SEARCH_PARAMETERS_INDEX.clone();
        let sub_filter = MemorySubscriptionFilter::new(
            &TenantId::System,
            &ProjectId::System,
            resolver.clone(),
            Subscription {
                criteria: Box::new(FHIRString {
                    value: Some("Patient?name=Smith".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let patient = Resource::Patient(Patient {
            name: Some(vec![Box::new(HumanName {
                family: Some(Box::new(FHIRString {
                    value: Some("Smith".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            })]),
            ..Default::default()
        });

        assert_eq!(sub_filter.matches(&patient).await.unwrap(), true);

        let sub_filter_partial = MemorySubscriptionFilter::new(
            &TenantId::System,
            &ProjectId::System,
            resolver.clone(),
            Subscription {
                criteria: Box::new(FHIRString {
                    value: Some("Patient?name=Sm".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(sub_filter_partial.matches(&patient).await.unwrap(), true);

        let sub_filter_casing = MemorySubscriptionFilter::new(
            &TenantId::System,
            &ProjectId::System,
            resolver.clone(),
            Subscription {
                criteria: Box::new(FHIRString {
                    value: Some("Patient?name=sm".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(sub_filter_casing.matches(&patient).await.unwrap(), true);

        let patient = Resource::Patient(Patient {
            name: Some(vec![Box::new(HumanName {
                family: Some(Box::new(FHIRString {
                    value: Some("NotSmith".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            })]),
            ..Default::default()
        });

        assert_eq!(sub_filter.matches(&patient).await.unwrap(), false);
    }
}
