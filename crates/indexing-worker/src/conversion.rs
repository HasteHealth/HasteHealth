/// Reference of conversions found here https://www.hl7.org/fhir/R4/search.html#table
use oxidized_fhir_model::r4::types::{
    FHIRDecimal, FHIRInteger, FHIRPositiveInt, FHIRString, FHIRUnsignedInt, SearchParameter,
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_reflect::MetaValue;

pub enum InsertableIndex {
    String(Vec<String>),
}

#[derive(OperationOutcomeError, Debug)]
enum InsertableIndexError {
    #[fatal(
        code = "exception",
        diagnostic = "Invalid type for insertable index: {arg0}"
    )]
    InvalidType(String),
    #[fatal(
        code = "exception",
        diagnostic = "Failed to downcast value to number: {arg0}"
    )]
    FailedDowncast(String),
}

// "http://hl7.org/fhirpath/System.String" => value
//     .as_any()
//     .downcast_ref::<String>()
//     .map(|v| vec![v.clone()])
//     .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string())),

fn convert_fp_string(value: &FHIRString) -> Vec<String> {
    value
        .value
        .as_ref()
        .map(|v| vec![v.to_string()])
        .unwrap_or_else(|| vec![])
}

fn convert_optional_fp_string(value: &Option<Box<FHIRString>>) -> Vec<String> {
    value
        .as_ref()
        .map(|v| convert_fp_string(v))
        .unwrap_or_else(|| vec![])
}

fn convert_optional_fp_string_vec(value: &Option<Vec<Box<FHIRString>>>) -> Vec<String> {
    value
        .as_ref()
        .map(|v| v.iter().flat_map(|s| convert_fp_string(s)).collect())
        .unwrap_or_else(|| vec![])
}

fn index_string(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.typename() {
        "FHIRString" => {
            let fp_string = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRString>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_string
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        // Even though spec states won't encounter this it does. [ImplementationGuide.description]
        "FHIRMarkdown" => {
            let fp_markdown = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRMarkdown>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_markdown
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "HumanName" => {
            let human_name = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::HumanName>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            let mut string_index = vec![];
            string_index.extend(convert_optional_fp_string(&human_name.text));
            string_index.extend(convert_optional_fp_string(&human_name.family));
            string_index.extend(convert_optional_fp_string_vec(&human_name.given));
            string_index.extend(convert_optional_fp_string_vec(&human_name.prefix));
            string_index.extend(convert_optional_fp_string_vec(&human_name.suffix));
            Ok(string_index)
        }
        "Address" => {
            let mut string_index = vec![];
            let address = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::Address>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            string_index.extend(convert_optional_fp_string(&address.text));
            string_index.extend(convert_optional_fp_string_vec(&address.line));
            string_index.extend(convert_optional_fp_string(&address.city));
            string_index.extend(convert_optional_fp_string(&address.district));
            string_index.extend(convert_optional_fp_string(&address.state));
            string_index.extend(convert_optional_fp_string(&address.postalCode));
            string_index.extend(convert_optional_fp_string(&address.country));

            Ok(string_index)
        }

        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_number(value: &dyn MetaValue) -> Result<f64, InsertableIndexError> {
    match value.typename() {
        "FHIRInteger" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            index_number(fp_integer.value.as_ref().unwrap_or(&0))
        }
        "FHIRDecimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            index_number(fp_decimal.value.as_ref().unwrap_or(&0.0))
        }
        "FHIRPositiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            index_number(fp_positive_int.value.as_ref().unwrap_or(&0))
        }
        "FHIRUnsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            index_number(fp_unsigned_int.value.as_ref().unwrap_or(&0))
        }
        "http://hl7.org/fhirpath/System.Integer" => value
            .as_any()
            .downcast_ref::<i64>()
            .map(|v| *v as f64)
            .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string())),

        "http://hl7.org/fhirpath/System.Decimal" => value
            .as_any()
            .downcast_ref::<f64>()
            .map(|v| *v)
            .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string())),
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

pub fn to_insertable_index(
    parameter: &SearchParameter,
    result: Vec<&dyn MetaValue>,
) -> Result<Vec<InsertableIndex>, OperationOutcomeError> {
    match parameter.type_.value.as_ref().map(|v| v.as_str()) {
        Some("number") => {
            panic!();
        }
        Some("composite") | Some("uri") | Some("reference") | Some("date") | Some("token")
        | Some("string") | Some("quantity") => {
            panic!()
        }
        Some(_) | None => Err(InsertableIndexError::InvalidType(
            parameter
                .type_
                .value
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or_else(|| "<unknown>")
                .to_string(),
        )
        .into()),
    }
}
