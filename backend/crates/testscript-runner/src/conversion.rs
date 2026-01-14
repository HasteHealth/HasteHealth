use haste_fhir_model::r4::generated::types::{
    FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRPositiveInt, FHIRUnsignedInt,
};
use haste_reflect::MetaValue;

#[derive(Debug, PartialEq)]
pub enum ConvertedValue {
    String(String),
    Boolean(bool),
    Number(f64),
}

fn downcast_string(value: &dyn MetaValue) -> Option<String> {
    match value.typename() {
        "FHIRCanonical" | "FHIRBase64Binary" | "FHIRCode" | "FHIRString" | "FHIROid"
        | "FHIRUri" | "FHIRUrl" | "FHIRUuid" | "FHIRXhtml" => {
            downcast_string(value.get_field("value").unwrap_or(&"".to_string()))
        }

        "http://hl7.org/fhirpath/System.String" => {
            value.as_any().downcast_ref::<String>().map(|v| v.clone())
        }
        _ => None,
    }
}

fn downcast_number(value: &dyn MetaValue) -> Option<f64> {
    match value.typename() {
        "FHIRInteger" => value
            .as_any()
            .downcast_ref::<FHIRInteger>()
            .and_then(|fp_int| downcast_number(fp_int.value.as_ref().unwrap_or(&0))),
        "FHIRDecimal" => value
            .as_any()
            .downcast_ref::<FHIRDecimal>()
            .and_then(|fp_dec| downcast_number(fp_dec.value.as_ref().unwrap_or(&(0 as f64)))),
        "FHIRPositiveInt" => value
            .as_any()
            .downcast_ref::<FHIRPositiveInt>()
            .and_then(|fp_pint| downcast_number(fp_pint.value.as_ref().unwrap_or(&(0 as u64)))),

        "FHIRUnsignedInt" => value
            .as_any()
            .downcast_ref::<FHIRUnsignedInt>()
            .and_then(|fp_uint| downcast_number(fp_uint.value.as_ref().unwrap_or(&(0 as u64)))),
        "http://hl7.org/fhirpath/System.Integer" => {
            value.as_any().downcast_ref::<i64>().map(|v| *v as f64)
        }

        "http://hl7.org/fhirpath/System.Decimal" => {
            value.as_any().downcast_ref::<f64>().map(|v| *v)
        }
        _ => None,
    }
}

fn downcast_bool(value: &dyn MetaValue) -> Option<bool> {
    match value.typename() {
        "http://hl7.org/fhirpath/System.Boolean" => {
            value.as_any().downcast_ref::<bool>().map(|v| *v)
        }

        "FHIRBoolean" => value
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .and_then(|fp_bool| downcast_bool(fp_bool.value.as_ref().unwrap_or(&false))),

        _ => None,
    }
}

pub fn convert_meta_value(value: &dyn MetaValue) -> Option<ConvertedValue> {
    if let Some(s) = downcast_string(value) {
        return Some(ConvertedValue::String(s));
    } else if let Some(i) = downcast_number(value) {
        return Some(ConvertedValue::Number(i));
    } else if let Some(b) = downcast_bool(value) {
        return Some(ConvertedValue::Boolean(b));
    }
    None
}

pub fn convert_string_value(value: &str) -> Option<ConvertedValue> {
    if value == "true" || value == "false" {
        if let Ok(b) = value.parse::<bool>() {
            return Some(ConvertedValue::Boolean(b));
        }
        None
    } else if let Ok(i) = value.parse::<f64>() {
        return Some(ConvertedValue::Number(i));
    } else {
        return Some(ConvertedValue::String(value.to_string()));
    }
}
