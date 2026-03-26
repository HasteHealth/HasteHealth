use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, DATE_TIME_TYPES, NUMBER_TYPES, PRIMITIVE_TYPES, STRING_TYPES, downcast_bool,
        downcast_number, downcast_string,
    },
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

#[derive(PartialEq, Debug)]
pub enum Primitive {
    Boolean(bool),
    Number(f64),
    String(String),
}

pub fn primitive_conversion(
    value: &dyn MetaValue,
) -> Result<Option<Primitive>, OperationOutcomeError> {
    let type_name = value.typename();
    if PRIMITIVE_TYPES.contains(type_name) {
        if STRING_TYPES.contains(type_name) {
            Ok(Some(Primitive::String(downcast_string(value).map_err(
                |e| {
                    OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        format!("Failed to downcast value to string: {}", e),
                    )
                },
            )?)))
        } else if NUMBER_TYPES.contains(type_name) {
            Ok(Some(Primitive::Number(downcast_number(value).map_err(
                |e| {
                    OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        format!("Failed to downcast value to number: {}", e),
                    )
                },
            )?)))
        } else if BOOLEAN_TYPES.contains(type_name) {
            Ok(Some(Primitive::Boolean(downcast_bool(value).map_err(
                |e| {
                    OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        format!("Failed to downcast value to boolean: {}", e),
                    )
                },
            )?)))
        } else if DATE_TIME_TYPES.contains(type_name) {
            Ok(Some(Primitive::String(downcast_string(value).map_err(
                |e| {
                    OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        format!("Failed to downcast value to string: {}", e),
                    )
                },
            )?)))
        } else {
            Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                format!("Unsupported primitive type: {}", type_name),
            ))
        }
    } else {
        Ok(None)
    }
}

pub fn check_bare_primitive_pattern(
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
