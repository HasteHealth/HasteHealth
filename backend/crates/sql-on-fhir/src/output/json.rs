use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use ordermap::OrderMap;

use crate::OutputResults;

pub fn json(
    results: Vec<OrderMap<String, OutputResults>>,
) -> Result<Vec<u8>, OperationOutcomeError> {
    let mut byte_vector = Vec::new();

    serde_json::to_writer(&mut byte_vector, &results).map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::PROCESSING,
            "Failed to write JSON output".to_string(),
        )
    })?;

    Ok(byte_vector)
}
