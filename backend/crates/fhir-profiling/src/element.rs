use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::resources::OperationOutcomeIssue;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Path;

use crate::FHIRProfileCTX;

pub async fn validate_element<'a>(
    _ctx: FHIRProfileCTX<'a, impl CanonicalResolver>,
    _element_pointer: Path,
    _value_pointer: Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    Ok(vec![])
}
