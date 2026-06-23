use haste_fhir_model::r4::{
    datetime::{Date, DateTime, Instant, Time},
    generated::{
        terminology::IssueType,
        types::{
            FHIRBase64Binary, FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDate, FHIRDateTime,
            FHIRDecimal, FHIRId, FHIRInstant, FHIRInteger, FHIRMarkdown, FHIROid, FHIRPositiveInt,
            FHIRString, FHIRTime, FHIRUnsignedInt, FHIRUri, FHIRUrl, FHIRUuid,
        },
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveValue {
    Boolean(bool),
    Number(f64),
    String(String),
}

fn downcast_meta_value<'a, T: 'static>(value: &'a dyn MetaValue) -> Option<&'a T> {
    value.as_any().downcast_ref::<T>().or_else(|| {
        value
            .as_any()
            .downcast_ref::<Box<T>>()
            .map(|boxed| boxed.as_ref())
    })
}

fn missing_value_error(type_name: &str) -> OperationOutcomeError {
    OperationOutcomeError::error(
        IssueType::Invalid(None),
        format!("{type_name} value is missing."),
    )
}

fn convert_with<T, R, F>(
    value: &dyn MetaValue,
    type_name: &str,
    extractor: F,
) -> Result<R, OperationOutcomeError>
where
    T: 'static,
    F: FnOnce(&T) -> Option<R>,
{
    downcast_meta_value::<T>(value)
        .and_then(extractor)
        .ok_or_else(|| missing_value_error(type_name))
}

pub fn convert_meta_value(value: &dyn MetaValue) -> Result<PrimitiveValue, OperationOutcomeError> {
    match value.fhir_type() {
        "instant" => convert_with::<FHIRInstant, _, _>(value, "instant", |primitive| {
            primitive
                .value
                .as_ref()
                .map(|instant| PrimitiveValue::String(instant.to_string()))
        }),
        "time" => convert_with::<FHIRTime, _, _>(value, "time", |primitive| {
            primitive
                .value
                .as_ref()
                .map(|time| PrimitiveValue::String(time.to_string()))
        }),
        "date" => convert_with::<FHIRDate, _, _>(value, "date", |primitive| {
            primitive
                .value
                .as_ref()
                .map(|date| PrimitiveValue::String(date.to_string()))
        }),
        "dateTime" => convert_with::<FHIRDateTime, _, _>(value, "dateTime", |primitive| {
            primitive
                .value
                .as_ref()
                .map(|date_time| PrimitiveValue::String(date_time.to_string()))
        }),
        "decimal" => convert_with::<FHIRDecimal, _, _>(value, "decimal", |primitive| {
            primitive.value.map(PrimitiveValue::Number)
        }),
        "boolean" => convert_with::<FHIRBoolean, _, _>(value, "boolean", |primitive| {
            primitive.value.map(PrimitiveValue::Boolean)
        }),
        "integer" => convert_with::<FHIRInteger, _, _>(value, "integer", |primitive| {
            primitive
                .value
                .map(|number| PrimitiveValue::Number(number as f64))
        }),
        "string" => convert_with::<FHIRString, _, _>(value, "string", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "uri" => convert_with::<FHIRUri, _, _>(value, "uri", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "base64Binary" => {
            convert_with::<FHIRBase64Binary, _, _>(value, "base64Binary", |primitive| {
                primitive
                    .value
                    .as_ref()
                    .cloned()
                    .map(PrimitiveValue::String)
            })
        }
        "code" => convert_with::<FHIRCode, _, _>(value, "code", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "id" => convert_with::<FHIRId, _, _>(value, "id", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "oid" => convert_with::<FHIROid, _, _>(value, "oid", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "unsignedInt" => convert_with::<FHIRUnsignedInt, _, _>(value, "unsignedInt", |primitive| {
            primitive
                .value
                .map(|number| PrimitiveValue::Number(number as f64))
        }),
        "positiveInt" => convert_with::<FHIRPositiveInt, _, _>(value, "positiveInt", |primitive| {
            primitive
                .value
                .map(|number| PrimitiveValue::Number(number as f64))
        }),
        "markdown" => convert_with::<FHIRMarkdown, _, _>(value, "markdown", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "url" => convert_with::<FHIRUrl, _, _>(value, "url", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "canonical" => convert_with::<FHIRCanonical, _, _>(value, "canonical", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "uuid" => convert_with::<FHIRUuid, _, _>(value, "uuid", |primitive| {
            primitive
                .value
                .as_ref()
                .cloned()
                .map(PrimitiveValue::String)
        }),
        "http://hl7.org/fhirpath/System.String" => value
            .as_any()
            .downcast_ref::<String>()
            .or_else(|| {
                value
                    .as_any()
                    .downcast_ref::<Box<String>>()
                    .map(|boxed| boxed.as_ref())
            })
            .cloned()
            .map(PrimitiveValue::String)
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.String")),
        "http://hl7.org/fhirpath/System.Boolean" => value
            .as_any()
            .downcast_ref::<bool>()
            .copied()
            .map(PrimitiveValue::Boolean)
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.Boolean")),
        "http://hl7.org/fhirpath/System.Integer" => value
            .as_any()
            .downcast_ref::<i64>()
            .copied()
            .map(|number| PrimitiveValue::Number(number as f64))
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.Integer")),
        "http://hl7.org/fhirpath/System.Decimal" => value
            .as_any()
            .downcast_ref::<f64>()
            .copied()
            .map(PrimitiveValue::Number)
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.Decimal")),
        "http://hl7.org/fhirpath/System.Date" => value
            .as_any()
            .downcast_ref::<Date>()
            .map(|date| PrimitiveValue::String(date.to_string()))
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.Date")),
        "http://hl7.org/fhirpath/System.DateTime" => value
            .as_any()
            .downcast_ref::<DateTime>()
            .map(|date_time| PrimitiveValue::String(date_time.to_string()))
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.DateTime")),
        "http://hl7.org/fhirpath/System.Instant" => value
            .as_any()
            .downcast_ref::<Instant>()
            .map(|instant| PrimitiveValue::String(instant.to_string()))
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.Instant")),
        "http://hl7.org/fhirpath/System.Time" => value
            .as_any()
            .downcast_ref::<Time>()
            .map(|time| PrimitiveValue::String(time.to_string()))
            .ok_or_else(|| missing_value_error("http://hl7.org/fhirpath/System.Time")),
        type_name => Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            format!("Unsupported primitive type: '{type_name}'"),
        )),
    }
}
