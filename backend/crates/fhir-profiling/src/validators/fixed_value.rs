use haste_fhir_model::r4::conversion::PRIMITIVE_TYPES;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

use crate::{slicing::items, validators::utilities};

/// Validates perfect match between fixed value and data.
/// Effectively this is a deep equality check between v1 and
pub fn is_equal(v1: &dyn MetaValue, v2: &dyn MetaValue) -> Result<bool, OperationOutcomeError> {
    if PRIMITIVE_TYPES.contains(v1.fhir_type()) {
        return Ok(utilities::primitive_conversion(v1)? == utilities::primitive_conversion(v2)?);
    }
    if v1.fhir_type() != v2.fhir_type() {
        return Ok(false);
    }
    for key in v1.fields() {
        let v1 = v1.get_field(key);
        let v2 = v2.get_field(key);

        if let Some(v1) = v1
            && let Some(v2) = v2
        {
            // Could be an array, so compare the values pairwise and confirm
            // the lengths agree.
            if items(v1).count() != items(v2).count() {
                return Ok(false);
            }
            for (v1_value, v2_value) in items(v1).zip(items(v2)) {
                if !is_equal(v1_value, v2_value)? {
                    return Ok(false);
                }
            }
        } else {
            // If one of the values is none verify that both are none in this case.
            if v1.is_some() != v2.is_some() {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use haste_fhir_model::r4::generated::types::{Address, FHIRString};

    use super::*;

    #[test]
    fn test_are_metavalues_equal() {
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

        assert!(!is_equal(&data, &pattern).unwrap());
    }
}
