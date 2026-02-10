use haste_fhir_model::r4::generated::resources::Resource;

pub trait SubscriptionFilter {
    fn matches(&self, resource: &Resource) -> bool;
}
