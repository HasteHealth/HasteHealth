use haste_fhir_model::r4::generated::{resources::StructureDefinition, types::ElementDefinition};
use haste_pointer::Pointer;

pub fn validate_element(
    element_pointer: Pointer<StructureDefinition, Vec<Box<ElementDefinition>>>,
) {
}
