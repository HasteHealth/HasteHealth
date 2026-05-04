pub mod providers;
pub mod structs;
pub mod traits;

fn extract_code_from_operation_definition(
    operation: &haste_fhir_model::r4::generated::resources::OperationDefinition,
) -> Option<(&str, &str)> {
}
