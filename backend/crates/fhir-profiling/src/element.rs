use haste_codegen::traversal;
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::OperationOutcomeIssue, terminology::IssueType, types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, Path};

use crate::FHIRProfileCTX;

pub async fn validate_element<'a>(
    ctx: FHIRProfileCTX<'a, impl CanonicalResolver>,
    element_pointer: Path,
    _value_pointer: Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let Some((elements_pointer, Key::Index(index))) = element_pointer.ascend() else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Invalid element path: {}", element_pointer),
        ));
    };

    let elements = elements_pointer
        .get::<Vec<Box<ElementDefinition>>>(ctx.root)
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Invalid elements path: {}", elements_pointer),
            )
        })?;

    let _children = traversal::ele_index_to_child_indices(elements, index)
        .map_err(|error| OperationOutcomeError::error(IssueType::Exception(None), error))?;

    let _element = element_pointer.get::<Box<ElementDefinition>>(ctx.root);

    Ok(vec![])
}
