use std::io::{BufReader, Read};

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

pub fn read_stdin(k: &mut String) -> Result<(), OperationOutcomeError> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin);

    reader.read_to_string(k).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to read from stdin: {}", e),
        )
    })?;

    Ok(())
}
