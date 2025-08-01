use oxidized_fhir_serialization_json::{FHIRJSONDeserializer, FHIRJSONSerializer};
use serde_json::Value;

use crate::DateTime;

impl FHIRJSONDeserializer for DateTime {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        todo!()
    }

    fn from_serde_value(
        v: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        todo!()
    }
}

impl FHIRJSONSerializer for DateTime {
    fn serialize_value(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn serialize_extension(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn is_fp_primitive(&self) -> bool {
        todo!()
    }
}

use crate::Date;

impl FHIRJSONDeserializer for Date {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        todo!()
    }

    fn from_serde_value(
        v: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        todo!()
    }
}

impl FHIRJSONSerializer for Date {
    fn serialize_value(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn serialize_extension(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn is_fp_primitive(&self) -> bool {
        todo!()
    }
}

use crate::Time;

impl FHIRJSONDeserializer for Time {
    fn from_json_str(
        s: &str,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        todo!()
    }

    fn from_serde_value(
        v: &Value,
        context: oxidized_fhir_serialization_json::Context,
    ) -> Result<Self, oxidized_fhir_serialization_json::errors::DeserializeError> {
        todo!()
    }
}

impl FHIRJSONSerializer for Time {
    fn serialize_value(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn serialize_extension(
        &self,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, oxidized_fhir_serialization_json::SerializeError> {
        todo!()
    }

    fn is_fp_primitive(&self) -> bool {
        todo!()
    }
}
