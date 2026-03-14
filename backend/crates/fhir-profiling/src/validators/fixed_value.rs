use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

use crate::validators::utilities;

/// Validates perfect match between fixed value and data.
/// Effectively this is a deep equality check between v1 and v2.
#[allow(dead_code)]
pub fn validate_fixed_value(
    v1: &dyn MetaValue,
    v2: &dyn MetaValue,
) -> Result<bool, OperationOutcomeError> {
    if v1.typename() != v2.typename() {
        return Ok(false);
    }

    let pattern_fields = v1.fields();

    if pattern_fields.len() == 0 {
        utilities::check_bare_primitive_pattern(v1, v2)
    } else {
        for key in pattern_fields {
            let v1 = v1.get_field(key);
            let v2 = v2.get_field(key);

            if v1.is_some() != v2.is_some() {
                return Ok(false);
            }

            if let Some(v1) = v1
                && let Some(v2) = v2
                && !validate_fixed_value(v1, v2)?
            {
                return Ok(false);
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
    fn test_are_metavalues_equal() {
        let pattern = Address {
            line: Some(vec![Box::new(FHIRString {
                value: Some("test".to_string()),
                ..Default::default()
            })]),
            ..Default::default()
        };

        let data = Address {
            line: Some(vec![Box::new(FHIRString {
                value: Some("test".to_string()),
                ..Default::default()
            })]),
            city: Some(Box::new(FHIRString {
                value: Some("any".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(!validate_fixed_value(&data, &pattern).unwrap());
    }
}
