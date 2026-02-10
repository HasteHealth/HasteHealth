use haste_fhir_model::r4::generated::{
    resources::{Resource, Subscription},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;

pub mod traits;

/// In memory representation of a subscription filter. This is what we will use to evaluate whether a given subscription matches an incoming event.
struct MemorySubscriptionFilter {}

impl TryFrom<Subscription> for MemorySubscriptionFilter {
    type Error = OperationOutcomeError;

    fn try_from(_value: Subscription) -> Result<Self, Self::Error> {
        Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "SubscriptionFilter conversion not implemented".to_string(),
        ))
    }
}

impl traits::SubscriptionFilter for MemorySubscriptionFilter {
    fn matches(&self, _resource: &Resource) -> bool {
        todo!("Not Implemented.")
    }
}
