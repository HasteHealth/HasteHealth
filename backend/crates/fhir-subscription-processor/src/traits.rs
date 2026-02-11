use haste_fhir_model::r4::generated::resources::Resource;
use haste_fhir_operation_error::OperationOutcomeError;

pub trait SubscriptionFilter {
    fn matches(
        &self,
        resource: &Resource,
    ) -> impl Future<Output = Result<bool, OperationOutcomeError>>;
}
