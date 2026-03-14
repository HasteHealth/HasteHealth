use haste_fhir_model::r4::{
    datetime::{Date, DateTime, Time},
    generated::terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

/**
 * 067  public static final String FP_String = "http://hl7.org/fhirpath/System.String";
 * 068  public static final String FP_Boolean = "http://hl7.org/fhirpath/System.Boolean";
 * 069  public static final String FP_Integer = "http://hl7.org/fhirpath/System.Integer";
 * 070  public static final String FP_Decimal = "http://hl7.org/fhirpath/System.Decimal";
 * 071  public static final String FP_Quantity = "http://hl7.org/fhirpath/System.Quantity";
 * 072  public static final String FP_DateTime = "http://hl7.org/fhirpath/System.DateTime";
 * 073  public static final String FP_Time = "http://hl7.org/fhirpath/System.Time";
 */

fn downcast_meta_value<'a, T: 'static>(
    value: &'a dyn MetaValue,
) -> Result<&'a T, OperationOutcomeError> {
    value.as_any().downcast_ref::<T>().ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            format!("Expected a value of type {}", std::any::type_name::<T>()),
        )
    })
}

fn check_bare_primitive_pattern(
    data_to_check: &dyn MetaValue,
    pattern: &dyn MetaValue,
) -> Result<bool, OperationOutcomeError> {
    match pattern.typename() {
        "http://hl7.org/fhirpath/System.String" => {
            let pattern_string = downcast_meta_value::<String>(pattern)?;
            let Ok(value_string) = downcast_meta_value::<String>(data_to_check) else {
                return Ok(false);
            };

            Ok(pattern_string == value_string)
        }
        "http://hl7.org/fhirpath/System.Boolean" => {
            let pattern_boolean = downcast_meta_value::<bool>(pattern)?;
            let Ok(value_boolean) = downcast_meta_value::<bool>(data_to_check) else {
                return Ok(false);
            };

            Ok(pattern_boolean == value_boolean)
        }
        "http://hl7.org/fhirpath/System.Integer" => {
            let pattern_integer = match pattern.type_id() == std::any::TypeId::of::<i64>() {
                true => *(downcast_meta_value::<i64>(pattern)?),
                false => *(downcast_meta_value::<u64>(pattern)?) as i64,
            };

            let value_integer = match data_to_check.type_id() == std::any::TypeId::of::<i64>() {
                true => *(downcast_meta_value::<i64>(data_to_check)?),
                false => *(downcast_meta_value::<u64>(data_to_check)?) as i64,
            };

            Ok(pattern_integer == value_integer)
        }
        "http://hl7.org/fhirpath/System.Decimal" => {
            let pattern_decimal = downcast_meta_value::<f64>(pattern)?;
            let Ok(value_decimal) = downcast_meta_value::<f64>(data_to_check) else {
                return Ok(false);
            };
            Ok(pattern_decimal == value_decimal)
        }

        "http://hl7.org/fhirpath/System.Date" => {
            let pattern_date = downcast_meta_value::<Date>(pattern)?;
            let Ok(value_date) = downcast_meta_value::<Date>(data_to_check) else {
                return Ok(false);
            };
            Ok(pattern_date == value_date)
        }

        "http://hl7.org/fhirpath/System.DateTime" => {
            let pattern_date = downcast_meta_value::<DateTime>(pattern)?;
            let Ok(value_date) = downcast_meta_value::<DateTime>(data_to_check) else {
                return Ok(false);
            };
            Ok(pattern_date == value_date)
        }

        "http://hl7.org/fhirpath/System.Time" => {
            let pattern_time = downcast_meta_value::<Time>(pattern)?;
            let Ok(value_time) = downcast_meta_value::<Time>(data_to_check) else {
                return Ok(false);
            };
            Ok(pattern_time == value_time)
        }

        _ => Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            format!("Unsupported pattern type: {}", pattern.typename()),
        )),
    }
}

#[allow(dead_code)]
pub fn validate_pattern(
    data_to_check: &dyn MetaValue,
    pattern: &dyn MetaValue,
) -> Result<bool, OperationOutcomeError> {
    if data_to_check.typename() != pattern.typename() {
        return Ok(false);
    }

    let pattern_fields = pattern.fields();

    if pattern_fields.len() == 0 {
        check_bare_primitive_pattern(data_to_check, pattern)
    } else {
        for key in pattern_fields {
            if let Some(pattern_value) = pattern.get_field(key) {
                let Some(data_value) = data_to_check.get_field(key) else {
                    return Ok(false);
                };

                if !validate_pattern(data_value, pattern_value)? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use haste_fhir_model::r4::generated::types::FHIRString;

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
}
