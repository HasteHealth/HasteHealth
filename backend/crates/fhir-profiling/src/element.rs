use haste_codegen::traversal;
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::OperationOutcomeIssue, terminology::IssueType, types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, Path};
use haste_reflect::MetaValue;

use crate::FHIRProfileCTX;

/**
 * Check if the element is constrained to profiles type.
 * @param element ElementDefinition to check
 * @param type The type found on the element.
 * @returns true|false as to whether the element is constrained to the type.
 */
fn validate_type_if_multiple_types_constrained<'a>(
    ctx: FHIRProfileCTX<'a, impl CanonicalResolver>,
    element: &ElementDefinition,
    type_: &str,
) -> bool {
    let Some(types) = &element.type_ else {
        return true;
    };

    if types
        .iter()
        .find(|t| t.code.value.as_ref().map(|s| s.as_str()) == Some(type_))
        .is_some()
    {
        true
    } else if type_ == "Element" {
        false
    } else {
        false
    }
}

fn _validate_cardinality(
    value_cardinality: usize,
    // Max could be '*' which is any number of elements.
    element_cardinality: (usize, Option<&str>),
) -> bool {
    false
}

fn validate_cardinality<'a>(
    ctx: FHIRProfileCTX<'a, impl CanonicalResolver>,
    element: &ElementDefinition,
    value: Option<&'a dyn MetaValue>,
) -> Vec<OperationOutcomeIssue> {
    match value {
        Some(v) => {
            v.flatten().len();
        }
        None => {}
    };

    vec![]
}

pub async fn validate_element<'a>(
    ctx: FHIRProfileCTX<'a, impl CanonicalResolver>,
    element_pointer: Path,
    value_pointer: Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let value = value_pointer.get(ctx.root);

    let Some((elements_pointer, Key::Index(index))) = element_pointer.ascend() else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Invalid element path: {}", element_pointer),
        ));
    };

    let elements = elements_pointer
        .get_typed::<Vec<Box<ElementDefinition>>>(ctx.root)
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Invalid elements path: {}", elements_pointer),
            )
        })?;

    let _children = traversal::ele_index_to_child_indices(elements, index)
        .map_err(|error| OperationOutcomeError::error(IssueType::Exception(None), error))?;

    let _element = element_pointer.get_typed::<Box<ElementDefinition>>(ctx.root);

    Ok(vec![])
}
