use haste_fhir_client::url::{ParsedParameter, ParsedParameters};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, Subscription},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;

pub mod traits;

#[allow(dead_code)]
pub struct SubscriptionParameter {
    fp_extract_expression: String,
    value: Vec<String>,
    modifier: Option<String>,
}

pub enum SubscriptionTrigger {
    // Based around simple Subscription.criteria.
    QueryFilter { resource_type: ResourceType, parameters: Vec<SubscriptionParameter> },
    // This could come from a subscriptiontopic which alllows arbitrary FHIRPath expressions, or from more complex criteria in the future.
    FHIRPathFilter { expression: String },
}

/// In memory representation of a subscription filter. This is what we will use to evaluate whether a given subscription matches an incoming event.
#[allow(dead_code)]
pub struct MemorySubscriptionFilter {
    triggers: Vec<SubscriptionTrigger>,
}

impl TryFrom<Subscription> for MemorySubscriptionFilter {
    type Error = OperationOutcomeError;

    fn try_from(value: Subscription) -> Result<Self, Self::Error> {
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

            let subscription_parsed_parameters = parsed_parameters
                .owned_parameters()
                .into_iter()
                .map(|parameter| match parameter {
                    ParsedParameter::Resource(resource_param) => {
                        let Some(search_parameter) =
                            haste_artifacts::search_parameters::get_search_parameter_for_name(
                                Some(&resource_type),
                                &resource_param.name,
                            )
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

                        Ok(SubscriptionParameter {
                            fp_extract_expression: fp_expression.clone(),
                            value: resource_param.value,
                            modifier: resource_param.modifier,
                        })

      
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
                })
                .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

            Ok(MemorySubscriptionFilter{
                triggers: vec![SubscriptionTrigger::QueryFilter { resource_type, parameters: subscription_parsed_parameters }],
            })

        } else {
            Err(OperationOutcomeError::error(
                IssueType::Exception(None),
                "SubscriptionFilter conversion not implemented".to_string(),
            ))
        }
    }
}

impl traits::SubscriptionFilter for MemorySubscriptionFilter {
    fn matches(&self, _resource: &Resource) -> bool {
        todo!("Not Implemented.")
    }
}


#[cfg(test)]    
mod tests {
    use haste_fhir_model::r4::generated::types::FHIRString;

    use super::*;
    
    #[test]
    fn quick_test_derive() {
        let subscription = Subscription {
                        criteria: Box::new(FHIRString {
                value: Some("Patient?name=Smith".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let sub_filter = MemorySubscriptionFilter::try_from(subscription).unwrap();

        assert_eq!(sub_filter.triggers.len(), 1);


        match &sub_filter.triggers[0] {
            SubscriptionTrigger::QueryFilter { resource_type, parameters } => {
                assert_eq!(resource_type, &ResourceType::Patient);
                assert_eq!(parameters[0].fp_extract_expression, "Patient.name");
                assert_eq!(parameters[0].value, vec!["Smith".to_string()]);
            }
            _ => panic!("Expected QueryFilter trigger"),
        };
        
    }

    #[test]
    fn modifier_check() {
        let subscription = Subscription {
                        criteria: Box::new(FHIRString {
                value: Some("Observation?category:missing=true".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let sub_filter = MemorySubscriptionFilter::try_from(subscription).unwrap();

        assert_eq!(sub_filter.triggers.len(), 1);


        match &sub_filter.triggers[0] {
            SubscriptionTrigger::QueryFilter { resource_type, parameters } => {
                assert_eq!(resource_type, &ResourceType::Observation);
                assert_eq!(parameters[0].fp_extract_expression, "Observation.category");
                assert_eq!(parameters[0].value, vec!["true".to_string()]);
                assert_eq!(parameters[0].modifier, Some("missing".to_string()));
            }
            _ => panic!("Expected QueryFilter trigger"),
        };
    }
}