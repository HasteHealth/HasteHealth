use crate::errors::DeserializeError;
use serde_json::Value;

pub trait FHIRJSONSerializer {
    fn serialize_value(&self) -> Option<String>;
    fn serialize_extension(&self) -> Option<String>;

    fn serialize_field(&self, field: &str) -> Option<String>;
    fn is_fp_primitive(&self) -> bool;
}

pub struct ContextAsField<'a> {
    pub field: &'a str,
    pub is_primitive: bool,
}

impl<'a> ContextAsField<'a> {
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

pub trait FHIRJSONDeserializer: Sized {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError>;
    fn from_serde_value(v: &Value, context: Context) -> Result<Self, DeserializeError>;
}

pub trait IsFHIRPrimitive {
    fn is_fp_primitive(&self) -> bool;
}
