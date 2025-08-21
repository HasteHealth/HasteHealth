use crate::errors::DeserializeError;
use crate::traits::{Context, FHIRJSONDeserializer};
use serde_json::Value;

fn get_value<'a>(value: &'a Value, context: &Context) -> Option<&'a Value> {
    match context {
        Context::AsValue => Some(value),
        Context::AsField(field_context) => value.get(field_context.field),
    }
}

impl FHIRJSONDeserializer for i64 {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        i64::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        let k = get_value(value, &context).and_then(|v| v.as_i64());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("i64".to_string()))
    }
}

impl FHIRJSONDeserializer for u64 {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        u64::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        let k = get_value(value, &context).and_then(|v| v.as_u64());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("u64".to_string()))
    }
}

impl FHIRJSONDeserializer for f64 {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        f64::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        let k = get_value(value, &context).and_then(|v| v.as_f64());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("f64".to_string()))
    }
}

impl FHIRJSONDeserializer for bool {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        bool::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        let k = get_value(value, &context).and_then(|v| v.as_bool());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("bool".to_string()))
    }
}

impl FHIRJSONDeserializer for String {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        String::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        let k = get_value(value, &context)
            .and_then(|v| v.as_str())
            .and_then(|s| Some(s.to_string()));
        k.ok_or_else(|| DeserializeError::FailedToConvertType("String".to_string()))
    }
}

impl<T> FHIRJSONDeserializer for Vec<T>
where
    T: FHIRJSONDeserializer,
{
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        Vec::<T>::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(v: &Value, context: Context) -> Result<Self, DeserializeError> {
        match &context {
            Context::AsValue => {
                if let Some(json_array) = v.as_array() {
                    json_array
                        .iter()
                        .map(|item| T::from_serde_value(item, Context::AsValue))
                        .collect()
                } else {
                    Err(DeserializeError::InvalidType(
                        "Expected an array".to_string(),
                    ))
                }
            }
            Context::AsField(field_context) => {
                if !field_context.is_primitive {
                    if let Some(json) = v.get(field_context.field)
                        && let Some(json_array) = json.as_array()
                    {
                        json_array
                            .iter()
                            .map(|item| T::from_serde_value(item, Context::AsValue))
                            .collect()
                    } else {
                        Err(DeserializeError::InvalidType(
                            "Expected an array".to_string(),
                        ))
                    }
                }
                // Special handling because array primitives live in two locations _<field> for element fields and <field> for values.
                else {
                    let mut return_v = vec![];
                    let values = {
                        if let Some(v) = v.get(field_context.field) {
                            if let Some(array) = v.as_array() {
                                Ok(Some(array))
                            } else {
                                Err(DeserializeError::InvalidType(
                                    "Expected an array".to_string(),
                                ))
                            }
                        } else {
                            Ok(None)
                        }
                    }?;
                    let elements = {
                        if let Some(v) = v.get(&format!("_{}", field_context.field)) {
                            if let Some(array) = v.as_array() {
                                Ok(Some(array))
                            } else {
                                Err(DeserializeError::InvalidType(
                                    "Expected an array".to_string(),
                                ))
                            }
                        } else {
                            Ok(None)
                        }
                    }?;

                    let length = std::cmp::max(
                        values.map(|v| v.len()).unwrap_or(0),
                        elements.map(|v| v.len()).unwrap_or(0),
                    );

                    for i in 0..length {
                        let mut json_v = serde_json::map::Map::new();
                        let value = values.and_then(|v| v.get(i));
                        let element = elements.and_then(|v| v.get(i));
                        if let Some(value) = value {
                            json_v.insert("fake_v".to_string(), value.clone());
                        }
                        if let Some(element) = element {
                            json_v.insert("_fake_v".to_string(), element.clone());
                        }
                        let res =
                            T::from_serde_value(&Value::Object(json_v), ("fake_v", true).into())?;
                        return_v.push(res);
                    }

                    Ok(return_v)
                }
            }
        }
    }
}

impl<T> FHIRJSONDeserializer for Option<T>
where
    T: FHIRJSONDeserializer,
{
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        Option::<T>::from_serde_value(&json_value, Context::AsValue)
    }

    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        match &context {
            Context::AsField(field_context) => match value.get(field_context.field) {
                Some(_v) => T::from_serde_value(value, context).map(|res| Some(res)),
                None => Ok(None),
            },
            Context::AsValue => {
                if value.is_null() {
                    Ok(None)
                } else {
                    T::from_serde_value(value, context).map(|res| Some(res))
                }
            }
        }
    }
}

impl<T> FHIRJSONDeserializer for Box<T>
where
    T: FHIRJSONDeserializer,
{
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let json_value: Value = serde_json::from_str(s)?;
        Box::<T>::from_serde_value(&json_value, Context::AsValue)
    }
    fn from_serde_value(value: &Value, context: Context) -> Result<Self, DeserializeError> {
        T::from_serde_value(value, context).map(|res| Box::new(res))
    }
}
