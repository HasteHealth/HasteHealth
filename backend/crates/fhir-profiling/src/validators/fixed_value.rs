use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

use crate::{slicing::items, validators::utilities};

/// Validates perfect match between fixed value and data.
/// Effectively this is a deep equality check between v1 and
pub fn is_equal(v1: &dyn MetaValue, v2: &dyn MetaValue) -> Result<bool, OperationOutcomeError> {
    if let Some(primitive) = utilities::primitive_conversion(v1)? {
        return Ok(Some(primitive) == utilities::primitive_conversion(v2)?);
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
    use haste_fhir_model::r4::generated::types::{
        Address, Coding, FHIRCode, FHIRDecimal, FHIRInteger, FHIRString, FHIRUri,
    };

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

    fn uri(value: &str) -> FHIRUri {
        FHIRUri {
            value: Some(value.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_uri_equality() {
        let a = uri("http://hl7.org/fhir/StructureDefinition/us-core-race");
        let b = uri("http://hl7.org/fhir/StructureDefinition/us-core-race");
        let c = uri("http://hl7.org/fhir/StructureDefinition/us-core-ethnicity");

        assert!(is_equal(&a, &b).unwrap());
        assert!(!is_equal(&a, &c).unwrap());
    }

    #[test]
    fn test_code_equals_uri_with_same_text() {
        let code = FHIRCode {
            value: Some("vital-signs".to_string()),
            ..Default::default()
        };

        assert!(is_equal(&code, &uri("vital-signs")).unwrap());
        assert!(is_equal(&uri("vital-signs"), &code).unwrap());
        assert!(!is_equal(&code, &uri("laboratory")).unwrap());
    }

    #[test]
    fn test_missing_string_value_equals_empty_string() {
        let missing = FHIRString::default();
        let empty = FHIRString {
            value: Some(String::new()),
            ..Default::default()
        };

        assert!(is_equal(&missing, &empty).unwrap());
        assert!(is_equal(&empty, &missing).unwrap());
    }

    #[test]
    fn test_integer_equals_decimal_with_same_number() {
        let integer = FHIRInteger {
            value: Some(5),
            ..Default::default()
        };
        let decimal = FHIRDecimal {
            value: Some(5.0),
            ..Default::default()
        };
        let other_decimal = FHIRDecimal {
            value: Some(5.5),
            ..Default::default()
        };

        assert!(is_equal(&integer, &decimal).unwrap());
        assert!(!is_equal(&integer, &other_decimal).unwrap());
    }

    #[test]
    fn test_complex_value_with_one_differing_field() {
        let coding = |code: &str| Coding {
            system: Some(Box::new(uri(
                "http://terminology.hl7.org/CodeSystem/observation-category",
            ))),
            code: Some(Box::new(FHIRCode {
                value: Some(code.to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(is_equal(&coding("vital-signs"), &coding("vital-signs")).unwrap());
        assert!(!is_equal(&coding("vital-signs"), &coding("laboratory")).unwrap());
    }
}
