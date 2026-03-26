use crate::r4::generated::types::{
    FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRPositiveInt, FHIRUnsignedInt,
};
use haste_reflect::MetaValue;
use std::{collections::HashSet, sync::LazyLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DowncastError {
    #[error("Failed to downcast value to type '{0}'")]
    FailedDowncast(String),
}

/// Number types to use in FHIR evaluation
pub static NUMBER_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRInteger");
    m.insert("FHIRDecimal");
    m.insert("FHIRPositiveInt");
    m.insert("FHIRUnsignedInt");
    m.insert("http://hl7.org/fhirpath/System.Decimal");
    m.insert("http://hl7.org/fhirpath/System.Integer");
    m
});

pub static BOOLEAN_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRBoolean");
    m.insert("http://hl7.org/fhirpath/System.Boolean");
    m
});

pub static DATE_TIME_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRDate");
    m.insert("FHIRDateTime");
    m.insert("FHIRInstant");
    m.insert("FHIRTime");
    m.insert("http://hl7.org/fhirpath/System.DateTime");
    m.insert("http://hl7.org/fhirpath/System.Instant");
    m.insert("http://hl7.org/fhirpath/System.Date");
    m.insert("http://hl7.org/fhirpath/System.Time");
    m
});

pub static STRING_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRBase64Binary");
    m.insert("FHIRCanonical");

    m.insert("FHIRCode");
    m.insert("FHIRString");
    m.insert("FHIROid");
    m.insert("FHIRUri");
    m.insert("FHIRUrl");
    m.insert("FHIRUuid");
    m.insert("FHIRXhtml");

    m.insert("http://hl7.org/fhirpath/System.String");
    m
});

pub fn downcast_bool(value: &dyn MetaValue) -> Result<bool, DowncastError> {
    match value.typename() {
        "http://hl7.org/fhirpath/System.Boolean" => value
            .as_any()
            .downcast_ref::<bool>()
            .map(|v| *v)
            .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string())),
        "FHIRBoolean" => {
            let fp_bool = value
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string()))?;
            downcast_bool(fp_bool.value.as_ref().unwrap_or(&false))
        }
        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}

pub fn downcast_string(value: &dyn MetaValue) -> Result<String, DowncastError> {
    match value.typename() {
        "FHIRCanonical" | "FHIRBase64Binary" | "FHIRCode" | "FHIRString" | "FHIROid"
        | "FHIRUri" | "FHIRUrl" | "FHIRUuid" | "FHIRXhtml" => {
            downcast_string(value.get_field("value").unwrap_or(&"".to_string()))
        }

        "http://hl7.org/fhirpath/System.String" => value
            .as_any()
            .downcast_ref::<String>()
            .map(|v| v.clone())
            .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string())),

        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}

pub fn downcast_number(value: &dyn MetaValue) -> Result<f64, DowncastError> {
    match value.typename() {
        "FHIRInteger" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string()))?;
            downcast_number(fp_integer.value.as_ref().unwrap_or(&0))
        }
        "FHIRDecimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string()))?;
            downcast_number(fp_decimal.value.as_ref().unwrap_or(&0.0))
        }
        "FHIRPositiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string()))?;

            downcast_number(fp_positive_int.value.as_ref().unwrap_or(&0))
        }
        "FHIRUnsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string()))?;

            downcast_number(fp_unsigned_int.value.as_ref().unwrap_or(&0))
        }
        "http://hl7.org/fhirpath/System.Integer" => value
            .as_any()
            .downcast_ref::<i64>()
            .map(|v| *v as f64)
            .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string())),

        "http://hl7.org/fhirpath/System.Decimal" => value
            .as_any()
            .downcast_ref::<f64>()
            .map(|v| *v)
            .ok_or_else(|| DowncastError::FailedDowncast(value.typename().to_string())),
        type_name => Err(DowncastError::FailedDowncast(type_name.to_string())),
    }
}
