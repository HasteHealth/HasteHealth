use crate::errors::DeserializeError;
use crate::traits::{Context, FHIRJSONDeserializer};
use sonic_rs::{JsonValueMutTrait, JsonValueTrait, Value, json};

fn get_value<'a>(value: &'a mut Value, context: &Context) -> Option<&'a mut Value> {
    match context {
        Context::AsValue => Some(value),
        Context::AsField(field_context) => value.get_mut(field_context.field),
    }
}

impl FHIRJSONDeserializer for i64 {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let mut json_value: Value = sonic_rs::from_str(s)?;
        i64::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let value = unsafe { &mut *(value as *mut Value) };
        let k = get_value(value, &context).and_then(|v| v.as_i64());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("i64".to_string()))
    }
}

impl FHIRJSONDeserializer for u64 {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let mut json_value: Value = sonic_rs::from_str(s)?;
        u64::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let value = unsafe { &mut *(value as *mut Value) };
        let k = get_value(value, &context).and_then(|v| v.as_u64());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("u64".to_string()))
    }
}

impl FHIRJSONDeserializer for f64 {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let mut json_value: Value = sonic_rs::from_str(s)?;
        f64::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let value = unsafe { &mut *(value as *mut Value) };
        let k = get_value(value, &context).and_then(|v| v.as_f64());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("f64".to_string()))
    }
}

impl FHIRJSONDeserializer for bool {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let mut json_value: Value = sonic_rs::from_str(s)?;
        bool::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let value = unsafe { &mut *(value as *mut Value) };
        let k = get_value(value, &context).and_then(|v| v.as_bool());
        k.ok_or_else(|| DeserializeError::FailedToConvertType("bool".to_string()))
    }
}

impl FHIRJSONDeserializer for String {
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let mut json_value: Value = sonic_rs::from_str(s)?;
        String::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let value = unsafe { &mut *(value as *mut Value) };
        let k = get_value(value, &context)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        k.ok_or_else(|| DeserializeError::FailedToConvertType("String".to_string()))
    }
}

impl<T> FHIRJSONDeserializer for Vec<T>
where
    T: FHIRJSONDeserializer,
{
    fn from_json_str(s: &str) -> Result<Self, DeserializeError> {
        let mut json_value: Value = sonic_rs::from_str(s)?;
        Vec::<T>::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(v: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let v = unsafe { &mut *(v as *mut Value) };
        match &context {
            Context::AsValue => {
                if let Some(json_array) = v.as_array_mut() {
                    json_array
                        .into_iter()
                        .map(|item| T::from_sonic_value(item, Context::AsValue))
                        .collect()
                } else {
                    Err(DeserializeError::InvalidType(
                        "Expected an array".to_string(),
                    ))
                }
            }
            Context::AsField(field_context) => {
                if !field_context.is_primitive {
                    if let Some(json) = v.get_mut(field_context.field)
                        && let Some(json_array) = json.as_array_mut()
                    {
                        json_array
                            .into_iter()
                            .map(|item| T::from_sonic_value(item, Context::AsValue))
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
                    let mut value_json = if let Some(v) = v.get_mut(field_context.field) {
                        Some(v.take())
                    } else {
                        None
                    };
                    let mut values = {
                        if let Some(value_json) = value_json.as_mut() {
                            if let Some(array) = value_json.as_array_mut() {
                                Ok(Some(array))
                            } else {
                                Err(DeserializeError::InvalidType(
                                    "Expected an array for values.".to_string(),
                                ))
                            }
                        } else {
                            Ok(None)
                        }
                    }?;

                    let mut elements_json =
                        if let Some(v) = v.get_mut(&format!("_{}", field_context.field)) {
                            Some(v.take())
                        } else {
                            None
                        };
                    let mut elements = {
                        if let Some(elements_json) = elements_json.as_mut() {
                            if let Some(array) = elements_json.as_array_mut() {
                                Ok(Some(array))
                            } else {
                                Err(DeserializeError::InvalidType(
                                    "Expected an array for elements.".to_string(),
                                ))
                            }
                        } else {
                            Ok(None)
                        }
                    }?;

                    let length = std::cmp::max(
                        values.as_ref().map(|v| v.len()).unwrap_or(0),
                        elements.as_ref().map(|v| v.len()).unwrap_or(0),
                    );

                    for i in 0..length {
                        let mut json_v: Value = json!({});
                        // Safe to unwrap: we just constructed it as an object.
                        let obj = json_v.as_object_mut().unwrap();

                        let value = values.as_mut().and_then(|v| v.get_mut(i));
                        let element = elements.as_mut().and_then(|v| v.get_mut(i));

                        if let Some(value) = value
                            && !value.is_null()
                        {
                            obj.insert(&"fake_v", std::mem::take(value));
                        }
                        if let Some(element) = element
                            && !element.is_null()
                        {
                            obj.insert(&"_fake_v", std::mem::take(element));
                        }

                        let res = T::from_sonic_value(&mut json_v, ("fake_v", true).into())?;
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
        let mut json_value: Value = sonic_rs::from_str(s)?;
        Option::<T>::from_sonic_value(&mut json_value, Context::AsValue)
    }

    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        let value = unsafe { &mut *(value as *mut Value) };
        match &context {
            Context::AsField(field_context) => match value.get(field_context.field) {
                Some(_v) => T::from_sonic_value(value, context).map(|res| Some(res)),
                None => Ok(None),
            },
            Context::AsValue => {
                if value.is_null() {
                    Ok(None)
                } else {
                    T::from_sonic_value(value, context).map(|res| Some(res))
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
        let mut json_value: Value = sonic_rs::from_str(s)?;
        Box::<T>::from_sonic_value(&mut json_value, Context::AsValue)
    }
    fn from_sonic_value(value: *mut Value, context: Context) -> Result<Self, DeserializeError> {
        T::from_sonic_value(value, context).map(|res| Box::new(res))
    }
}
