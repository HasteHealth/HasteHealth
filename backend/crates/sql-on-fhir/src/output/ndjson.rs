use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use ordermap::OrderMap;

use crate::OutputResults;

pub fn ndjson(
    results: Vec<OrderMap<String, OutputResults>>,
) -> Result<Vec<u8>, OperationOutcomeError> {
    let mut byte_vector = Vec::new();

    for result in results {
        serde_json::to_writer(&mut byte_vector, &result).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::PROCESSING,
                "Failed to write NDJSON output".to_string(),
            )
        })?;
        byte_vector.push(b'\n');
    }

    Ok(byte_vector)
}
