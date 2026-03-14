use haste_codegen::traversal;
use haste_fhir_model::r4::generated::{terminology::IssueType, types::ElementDefinition};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

fn is_slice(element: &ElementDefinition) -> bool {
    element.slicing.is_some()
}

pub struct SlicingDescriptor {
    /// The index of the element definition that contains the discriminator.
    #[allow(dead_code)]
    discriminator: usize,
    /// The indices of the slice element definitions that belong to the discriminator. The discriminator element is not included in this list.
    #[allow(dead_code)]
    slices: Vec<usize>,
}

#[allow(dead_code)]
/// Return child elements that are slice elemenet definitions.
pub fn get_slice_element_definition_locations(
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

#[allow(dead_code)]
struct SliceSplit {}

#[allow(dead_code)]
async fn split_values_into_slices(
    _elements: &Vec<ElementDefinition>,
    _slicing_descriptor: SlicingDescriptor,
    _values: Vec<&dyn MetaValue>,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}

/// The element discriminator specifies a path that is used to discriminate slices with.
/// However to know what the dicriminator should expect you need to use the element for the given discriminators path.
/// For example on US-core Patient has slicing like
/// ```json
// "slicing" : {
//   "discriminator" : [
//             {
//               "type" : "value",
//               "path" : "url"
//             }
//   ],
/// ```
///
/// For the race you would then look for Element.url and the fixed uri value.
///
#[allow(dead_code)]
fn find_element_definition_for_discriminator() -> Result<(), OperationOutcomeError> {
    Ok(())
}
