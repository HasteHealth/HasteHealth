use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
};

use crate::conversions::primitives::PrimitiveValue;

pub fn csv(
    results: Vec<BTreeMap<String, Vec<Option<PrimitiveValue>>>>,
) -> Result<Vec<u8>, OperationOutcomeError> {
    let byte_vector = Vec::new();
    let mut writer = BufWriter::new(byte_vector);
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
            let value_str = String::new();
            if let Some(values) = result.remove(key) {
                for value in values {
                    let value_str = match value {
                        Some(PrimitiveValue::Boolean(b)) => b.to_string(),
                        Some(PrimitiveValue::Number(n)) => n.to_string(),
                        Some(PrimitiveValue::String(s)) => s.clone(),
                        _ => "".to_string(),
                    };
                }
            }

            row.push(value_str);
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

    let output = writer.into_inner().map_err(|e| {
        OperationOutcomeError::error(
            IssueType::Processing(None),
            format!("Failed to write CSV output: {}", e),
        )
    })?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use csv;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename = "name", serialize_with = "serialize_vec_as_csv")]
        name: Vec<String>,
    }

    fn serialize_vec_as_csv<S>(vec: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let joined = vec.join(";");
        serializer.serialize_str(&joined)
    }

    #[test]
    fn test_many_csv() {
        let value = TestStruct {
            name: vec!["Alice".to_string(), "Bob".to_string()],
        };
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.serialize(&value).unwrap();
        let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        assert_eq!(data, "name\nAlice;Bob\n");
    }
}
