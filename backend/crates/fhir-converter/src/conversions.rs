use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, NUMBER_TYPES, STRING_TYPES, downcast_bool, downcast_number, downcast_string,
    },
    generated::{
        resources::Resource,
        types::{FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRString},
    },
};
use haste_reflect::MetaValue;
use liquid::{Error, Object, model::KString};
use liquid_core::Value;

/// Convert a liquid `Value` to FHIR context entries.
///
/// Scalars map to a single FHIR primitive. Arrays are flattened so each
/// element becomes a separate context value, matching the FHIRPath collection
/// model. Objects become a lazy `BackboneElement`-typed `MetaObject`. Nil
/// produces an empty context.
pub fn liquid_to_metavalue(value: Value) -> Result<Vec<Box<dyn MetaValue + Send + Sync>>, Error> {
    match value {
        Value::Scalar(s) => {
            // Bool must be checked before integer/float since liquid booleans
            // can round-trip through to_integer (true → 1).
            if let Some(b) = s.to_bool() {
                Ok(vec![Box::new(FHIRBoolean {
                    value: Some(b),
                    ..Default::default()
                })])
            } else if let Some(i) = s.to_integer() {
                Ok(vec![Box::new(FHIRInteger {
                    value: Some(i),
                    ..Default::default()
                })])
            } else if let Some(f) = s.to_float() {
                Ok(vec![Box::new(FHIRDecimal {
                    value: Some(f),
                    ..Default::default()
                })])
            } else {
                Ok(vec![Box::new(FHIRString {
                    value: Some(s.into_string().to_string()),
                    ..Default::default()
                })])
            }
        }
        Value::Array(arr) => arr
            .into_iter()
            .map(liquid_to_metavalue)
            .flat_map(|result| match result {
                Ok(vec) => vec.into_iter().map(|item| Ok(item)).collect(),
                Err(er) => vec![Err(er)],
            })
            .collect::<Result<Vec<Box<dyn MetaValue + Send + Sync>>, Error>>(),
        Value::Object(obj) => {
            println!("Converting object: {:?}", obj);
            let k: Resource = serde_json::from_value(
                serde_json::to_value(&obj).map_err(|e| Error::with_msg(format!("{}", e)))?,
            )
            .map_err(|e| {
                println!("{}", e);
                Error::with_msg(
                    "Must be a valid resource type to use fhirpath filter on.".to_string(),
                )
            })?;

            Ok(vec![Box::new(k)])
        }
        _ => Err(Error::with_msg(
            "Unsupported liquid value type for conversion to FHIR MetaValue",
        )),
    }
}

/// Convert a `MetaValue` to a liquid `Value`.
///
/// Primitive FHIR types are dispatched via `fhir_type()` and converted to
/// the matching liquid scalar. Complex types are recursively converted to
/// `Value::Object` using the same `flatten()` traversal that the FHIRPath
/// engine uses, so field semantics are consistent.
pub fn fhir_to_liquid(value: &dyn MetaValue) -> Value {
    let fhir_type = value.fhir_type();

    if NUMBER_TYPES.contains(fhir_type) {
        if let Ok(n) = downcast_number(value) {
            return Value::scalar(n);
        }
    }
    if BOOLEAN_TYPES.contains(fhir_type) {
        if let Ok(b) = downcast_bool(value) {
            return Value::scalar(b);
        }
    }
    if STRING_TYPES.contains(fhir_type) {
        if let Ok(s) = downcast_string(value) {
            return Value::scalar(s);
        }
    }

    // Complex type: build a liquid Object from the reflected fields.
    // fields() always returns &'static str so KString::from_static is free.
    // flatten() mirrors the FHIRPath engine's own traversal strategy:
    //   - single-valued fields  → flatten() yields [self]
    //   - collection fields     → flatten() yields each element
    let fields = value.fields();
    if fields.is_empty() {
        return Value::scalar(format!("{:?}", value));
    }

    let mut obj = Object::new();
    for field in fields {
        let Some(field_val) = value.get_field(field) else {
            continue;
        };
        let items: Vec<&dyn MetaValue> = field_val.flatten();
        let converted = match items.len() {
            0 => continue,
            _ => Value::Array(items.into_iter().map(fhir_to_liquid).collect()),
        };
        obj.insert(KString::from_static(field), converted);
    }
    Value::Object(obj)
}
