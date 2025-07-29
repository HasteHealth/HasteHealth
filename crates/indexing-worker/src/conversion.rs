use oxidized_fhir_model::r4::types::{
    FHIRDecimal, FHIRInteger, FHIRPositiveInt, FHIRUnsignedInt, SearchParameter,
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

fn index_string(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.typename() {
        "FHIRBase64Binary" => {
            let fp_base64 = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRBase64Binary>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_base64.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIRCanonical" => {
            let fp_canonical = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRCanonical>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_canonical
                .value
                .map(|v| vec![v])
                .unwrap_or_else(|| vec![]))
        }

        "FHIRCode" => {
            let fp_code = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRCode>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_code.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIRString" => {
            let fp_string = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRString>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_string.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIROid" => {
            let fp_oid = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIROid>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_oid.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIRUri" => {
            let fp_uri = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRUri>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_uri.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIRUrl" => {
            let fp_url = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRUrl>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_url.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIRUuid" => {
            let fp_uuid = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRUuid>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_uuid.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "FHIRXhtml" => {
            let fp_xhtml = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRXhtml>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_xhtml.value.map(|v| vec![v]).unwrap_or_else(|| vec![]))
        }
        "http://hl7.org/fhirpath/System.String" => value
            .as_any()
            .downcast_ref::<String>()
            .map(|v| vec![v.clone()])
            .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string())),

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
