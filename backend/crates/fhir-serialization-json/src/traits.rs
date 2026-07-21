use std::io::BufWriter;

use crate::errors::DeserializeError;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializeError {
    #[error("Serialization error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializeError(#[from] std::io::IntoInnerError<BufWriter<Vec<u8>>>),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Trait for serializing FHIR values to JSON.
pub trait FHIRJSONSerializer {
    /// Serializes the JSON value representation.
    ///
    /// Returns `true` if any output was written.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError`] if writing to `writer` fails or if the value
    /// cannot be serialized.
    fn serialize_value(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError>;

    /// Serializes the JSON extension representation.
    ///
    /// Returns `true` if any output was written.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError`] if writing to `writer` fails or if the
    /// extension cannot be serialized.
    fn serialize_extension(&self, writer: &mut dyn std::io::Write) -> Result<bool, SerializeError>;

    /// Serializes a named JSON field.
    ///
    /// Returns `true` if the field was written.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeError`] if writing to `writer` fails or if the field
    /// cannot be serialized.
    fn serialize_field(
        &self,
        field: &str,
        writer: &mut dyn std::io::Write,
    ) -> Result<bool, SerializeError>;

    /// Returns whether this value is a FHIR primitive.
    fn is_fp_primitive(&self) -> bool;
}

pub struct ContextAsField<'a> {
    pub field: &'a str,
    pub is_primitive: bool,
}

impl<'a> ContextAsField<'a> {
    #[must_use]
    pub fn new(field: &'a str, is_primitive: bool) -> Self {
        ContextAsField {
            field,
            is_primitive,
        }
    }
}

pub enum Context<'a> {
    AsField(ContextAsField<'a>),
    AsValue,
}

impl<'a> From<(&'a str, bool)> for Context<'a> {
    fn from(value: (&'a str, bool)) -> Self {
        Context::AsField(ContextAsField::new(value.0, value.1))
    }
}

impl<'a> From<(&'a String, bool)> for Context<'a> {
    fn from(value: (&'a String, bool)) -> Self {
        Context::AsField(ContextAsField::new(value.0.as_str(), value.1))
    }
}

/// Trait for deserializing FHIR values from JSON.
pub trait FHIRJSONDeserializer: Sized {
    /// Deserializes a value from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`DeserializeError`] if the input is not valid JSON or cannot be
    /// deserialized into `Self`.
    fn from_json_str(s: &str) -> Result<Self, DeserializeError>;

    /// Deserializes a value from a parsed JSON value.
    ///
    /// # Errors
    ///
    /// Returns [`DeserializeError`] if the JSON value cannot be deserialized
    /// into `Self` or if the provided context is invalid for deserialization.
    fn from_serde_value(v: *mut Value, context: Context) -> Result<Self, DeserializeError>;
}

pub trait IsFHIRPrimitive {
    fn is_fp_primitive(&self) -> bool;
}
