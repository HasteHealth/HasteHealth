use haste_fhir_model::r4::generated::{resources::StructureDefinition, types::ElementDefinition};
use haste_pointer::TypedPointer;

pub fn validate_element(
    element_pointer: TypedPointer<StructureDefinition, Vec<Box<ElementDefinition>>>,
) {
}
