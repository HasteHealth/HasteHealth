use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use ordermap::OrderMap;
use std::fmt::Write as FmtWrite;

use crate::{OutputResults, conversions::primitives::PrimitiveValue};

fn append_primitive_value(buffer: &mut String, value: &PrimitiveValue) {
    match value {
        PrimitiveValue::Boolean(b) => {
            let _ = write!(buffer, "{b}");
        }
        PrimitiveValue::Number(n) => {
            let _ = write!(buffer, "{n}");
        }
        PrimitiveValue::String(s) => {
            buffer.push_str(s);
        }
    }
}

pub fn csv(
    results: Vec<OrderMap<String, OutputResults>>,
) -> Result<Vec<u8>, OperationOutcomeError> {
    let mut byte_vector = Vec::new();
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(&mut byte_vector);
    let mut column_names = Vec::new();

    if let Some(header_col) = results.get(0) {
        column_names = header_col.keys().cloned().collect();

        writer.write_record(column_names.iter()).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::PROCESSING,
                "Failed to write CSV header".to_string(),
            )
        })?;
    }

    let mut row = csv::StringRecord::new();
    let mut cell_buffer = String::new();

    for result in &results {
        row.clear();

        for key in column_names.iter() {
            cell_buffer.clear();

            if let Some(value) = result.get(key) {
                let mut first = true;
                match value {
                    OutputResults::Scalar(singular_value) => {
                        if let Some(value) = singular_value {
                            append_primitive_value(&mut cell_buffer, value);
                        }
                    }
                    OutputResults::Collection(values) => {
                        for value in values.iter().flatten() {
                            if !first {
                                cell_buffer.push(';');
                            }
                            append_primitive_value(&mut cell_buffer, value);
                            first = false;
                        }
                    }
                }
            }

            row.push_field(&cell_buffer);
        }

        writer.write_record(&row).map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::PROCESSING,
                "Failed to write CSV row".to_string(),
            )
        })?;
    }

    writer.flush().map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::PROCESSING,
            "Failed to flush CSV output".to_string(),
        )
    })?;

    // Ensure all buffered bytes are written before returning the vector.
    drop(writer);

    Ok(byte_vector)
}
