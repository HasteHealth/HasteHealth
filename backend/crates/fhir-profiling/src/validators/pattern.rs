use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

use crate::{slicing::items, validators::utilities};

pub fn validate_pattern(
    value: &dyn MetaValue,
    pattern: &dyn MetaValue,
) -> Result<bool, OperationOutcomeError> {
    if value.fhir_type() != pattern.fhir_type() {
        return Ok(false);
    }

    let pattern_fields = pattern.fields();

    if pattern_fields.is_empty() {
        utilities::check_bare_primitive_pattern(value, pattern)
    } else {
        for key in &pattern_fields {
            if let Some(pattern_value) = pattern.get_field(key) {
                let Some(data_value) = value.get_field(key) else {
                    return Ok(false);
                };

                if items(pattern_value).count() > items(data_value).count() {
                    return Ok(false);
                }

                // Every value the pattern lists must be matched by some value
                // of the data, in any order.
                for pattern_value in items(pattern_value) {
                    let found = items(data_value)
                        .any(|v| validate_pattern(v, pattern_value).unwrap_or(false));

                    if !found {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use haste_fhir_model::r4::generated::types::{Address, FHIRString};

    use super::*;

    #[test]
    fn test_validate_pattern() {
        let pattern = "test".to_string();
        let data = "test".to_string();
        assert!(validate_pattern(&data, &pattern).unwrap());

        let pattern: u64 = 42;
        let data: u64 = 42;
        assert!(validate_pattern(&data, &pattern).unwrap());
    }

    #[test]
    fn test_complex_pattern() {
        use haste_fhir_model::r4::generated::types::CodeableConcept;

        let pattern = CodeableConcept {
            coding: None,
            text: Some(Box::new(FHIRString {
                value: Some("test".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        let data = CodeableConcept {
            coding: None,
            text: Some(Box::new(FHIRString {
                value: Some("test".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(validate_pattern(&data, &pattern).unwrap());

        let data2 = CodeableConcept {
            coding: None,
            text: Some(Box::new(FHIRString {
                value: Some("not-valid".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(!validate_pattern(&data2, &pattern).unwrap());
    }

    #[test]
    fn test_partial_pattern() {
        let pattern = Address {
            line: Some(vec![FHIRString {
                value: Some("test".to_string()),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let data = Address {
            line: Some(vec![FHIRString {
                value: Some("test".to_string()),
                ..Default::default()
            }]),
            city: Some(Box::new(FHIRString {
                value: Some("any".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(validate_pattern(&data, &pattern).unwrap());
    }
}
