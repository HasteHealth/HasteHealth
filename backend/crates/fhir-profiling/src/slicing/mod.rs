use std::sync::Arc;

use haste_codegen::{traversal, utilities::extract::field_name};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::OperationOutcomeIssue, terminology::IssueType, types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Path;
use haste_reflect::MetaValue;

use crate::FHIRProfileCTX;

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
/// Return child elements that are slice element definitions.
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

/// The discriminator element specifies a path from which to compare with.
/// To know how split should be done though we need the constant pattern etc... from that path.
/// For example Extension.url could be the discriminator, but
/// We need to pull from for example https://build.fhir.org/ig/HL7/US-Core/StructureDefinition-us-core-race.html
///  the actual value of the pattern to know how to split the slice. Which would be "http://hl7.org/fhir/us/core/StructureDefinition/us-core-race"
#[allow(dead_code)]
fn find_element_definition_for_discriminator<'a>(
    _ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    _discriminator_element: &ElementDefinition,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}

/// Returns all the slice locs that are relevant to the given discriminator.
fn get_slice_value_locs(
    discriminator_element: &ElementDefinition,
    value: &dyn MetaValue,
    value_path: &Path,
) -> Result<Vec<Path>, OperationOutcomeError> {
    let field = field_name(
        discriminator_element
            .path
            .value
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(""),
    );

    let slice_path = value_path.descend(&field);

    let Some(v) = slice_path.get(value) else {
        return Ok(vec![]);
    };

    if v.is_many() {
        Ok(v.flatten()
            .iter()
            .enumerate()
            .map(|(i, _)| slice_path.descend(&format!("{}", i)))
            .collect())
    } else {
        Ok(vec![slice_path])
    }
}

#[allow(dead_code)]
pub fn validate_slicing_descriptor<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    slicing_descriptor: &SlicingDescriptor,
    value: &dyn MetaValue,
    value_path: &Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let discriminator_element = ctx
        .profile()
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.element.get(slicing_descriptor.discriminator))
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!(
                    "Invalid slicing discriminator index: {}",
                    slicing_descriptor.discriminator
                ),
            )
        })?;

    let _slice_value_locs = get_slice_value_locs(discriminator_element, value, value_path)?;

    Ok(vec![])
}
