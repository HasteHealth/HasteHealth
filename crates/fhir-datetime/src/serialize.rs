use crate::{Date, DateTime, Instant, Time, parse_date, parse_datetime, parse_instant, parse_time};
use oxidized_fhir_serialization_json::errors::DeserializeError;
use oxidized_fhir_serialization_json::{Context, SerializeError};
use oxidized_fhir_serialization_json::{FHIRJSONDeserializer, FHIRJSONSerializer};
use serde_json::Value;

fn get_value<'a>(value: &'a Value, context: &Context) -> Option<&'a Value> {
    match context {
        Context::AsValue => Some(value),
        Context::AsField(field_context) => value.get(field_context.field),
    }
}

impl FHIRJSONDeserializer for DateTime {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        DateTime::from_serde_value(&json_value, Context::AsValue)
    }

    fn from_serde_value(
        value: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let k = get_value(value, &context)
            .and_then(|v| v.as_str().and_then(|v| parse_datetime(v).ok()));
        k.ok_or_else(|| DeserializeError::FailedToConvertType("DateTime".to_string()))
    }
}

impl FHIRJSONSerializer for DateTime {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(&[b'"'])?;
        writer.write_all(self.to_string().as_bytes())?;
        writer.write_all(&[b'"'])?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write_all("\"".as_bytes())?;
        writer.write_all(field.as_bytes())?;
        writer.write_all("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONDeserializer for Date {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        Date::from_serde_value(&json_value, Context::AsValue)
    }

    fn from_serde_value(
        value: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let k =
            get_value(value, &context).and_then(|v| v.as_str().and_then(|v| parse_date(v).ok()));
        k.ok_or_else(|| DeserializeError::FailedToConvertType("Date".to_string()))
    }
}

impl FHIRJSONSerializer for Date {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(&[b'"'])?;
        writer.write_all(self.to_string().as_bytes())?;
        writer.write_all(&[b'"'])?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write_all("\"".as_bytes())?;
        writer.write_all(field.as_bytes())?;
        writer.write_all("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONDeserializer for Time {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        Time::from_serde_value(&json_value, Context::AsValue)
    }

    fn from_serde_value(
        value: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let k =
            get_value(value, &context).and_then(|v| v.as_str().and_then(|v| parse_time(v).ok()));
        k.ok_or_else(|| DeserializeError::FailedToConvertType("Time".to_string()))
    }
}

impl FHIRJSONSerializer for Time {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(&[b'"'])?;
        writer.write_all(self.to_string().as_bytes())?;
        writer.write_all(&[b'"'])?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write_all("\"".as_bytes())?;
        writer.write_all(field.as_bytes())?;
        writer.write_all("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}

impl FHIRJSONDeserializer for Instant {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        Instant::from_serde_value(&json_value, Context::AsValue)
    }

    fn from_serde_value(
        value: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        let k =
            get_value(value, &context).and_then(|v| v.as_str().and_then(|v| parse_instant(v).ok()));
        k.ok_or_else(|| DeserializeError::FailedToConvertType("Instant".to_string()))
    }
}

impl FHIRJSONSerializer for Instant {
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError> {
        writer.write_all(&[b'"'])?;
        writer.write_all(self.to_string().as_bytes())?;
        writer.write_all(&[b'"'])?;

        Ok(true)
    }

    fn serialize_extension(
        &self,
        _writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        Ok(false)
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError> {
        writer.write_all("\"".as_bytes())?;
        writer.write_all(field.as_bytes())?;
        writer.write_all("\":".as_bytes())?;
        self.serialize_value(writer)?;

        Ok(true)
    }

    fn is_fp_primitive(&self) -> bool {
        false
    }
}
