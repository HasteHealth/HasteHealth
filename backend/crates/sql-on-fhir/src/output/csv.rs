use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use ordermap::OrderMap;
use std::io::{BufWriter, Write};

use crate::conversions::primitives::PrimitiveValue;

pub fn csv(
    results: Vec<OrderMap<String, Vec<Option<PrimitiveValue>>>>,
) -> Result<Vec<u8>, OperationOutcomeError> {
    let mut byte_vector = Vec::new();
    let mut writer = BufWriter::new(&mut byte_vector);
    let mut column_names = vec![];

    if let Some(header_col) = results.get(0) {
        column_names = header_col.keys().cloned().collect();
        let header = column_names.join(",");
        let header_bytes = header.as_bytes();

        writer.write(header_bytes).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Processing(None),
                "Failed to write CSV header".to_string(),
            )
        })?;
        writer.write(b"\n").map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Processing(None),
                "Failed to write CSV header".to_string(),
            )
        })?;
    }

    for mut result in results.into_iter() {
        let mut row: Vec<String> = Vec::new();

        for key in column_names.iter() {
            let mut value_strings = vec![];
            if let Some(values) = result.remove(key) {
                for value in values {
                    match value {
                        Some(PrimitiveValue::Boolean(b)) => value_strings.push(b.to_string()),
                        Some(PrimitiveValue::Number(n)) => value_strings.push(n.to_string()),
                        Some(PrimitiveValue::String(s)) => value_strings.push(s),
                        _ => {
                            // Do nothing
                        }
                    }
                }
            }

            row.push(value_strings.join(";"));
        }

        let row_str = row.join(",");
        let row_bytes = row_str.as_bytes();
        writer.write(row_bytes).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Processing(None),
                "Failed to write CSV row".to_string(),
            )
        })?;
        writer.write(b"\n").map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Processing(None),
                "Failed to write CSV row".to_string(),
            )
        })?;
    }

    drop(writer);

    Ok(byte_vector)
}
