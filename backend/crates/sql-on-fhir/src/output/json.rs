use crate::Row;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

pub fn json(results: &[Row]) -> Result<Vec<u8>, OperationOutcomeError> {
    let mut byte_vector = Vec::new();

    serde_json::to_writer(&mut byte_vector, &results).map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::processing(),
            "Failed to write JSON output".to_string(),
        )
    })?;

    Ok(byte_vector)
}
