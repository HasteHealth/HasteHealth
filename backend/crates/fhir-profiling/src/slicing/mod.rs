use haste_codegen::traversal;
use haste_fhir_model::r4::generated::{
    resources::StructureDefinition, terminology::IssueType, types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, Path};

fn is_slice(element: &ElementDefinition) -> bool {
    element.slicing.is_some()
}

pub struct SlicingDescriptor {
    discriminator: usize,
    slices: Vec<usize>,
}

pub fn get_slice_indices(
    elements: &[Box<ElementDefinition>],
    index: usize,
) -> Result<Vec<SlicingDescriptor>, OperationOutcomeError> {
    let children = traversal::ele_index_to_child_indices(elements, index)
        .map_err(|error| OperationOutcomeError::error(IssueType::Exception(None), error))?;

    let mut i = 0;

    let mut slice_indices = vec![];

    while i < children.len() {
        let child_index = children[i];
        let element = &elements[child_index];
        i += 1;

        if is_slice(element.as_ref()) {
            let mut slice_index = SlicingDescriptor {
                discriminator: child_index,
                slices: vec![],
            };

            while i < children.len()
                && elements[children[i]]
                    .sliceName
                    .as_ref()
                    .and_then(|v| v.value.as_ref())
                    .is_some()
            {
                slice_index.slices.push(children[i]);
                i += 1;
            }

            slice_indices.push(slice_index);
        }
    }

    Ok(slice_indices)
}
