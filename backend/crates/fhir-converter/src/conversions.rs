use std::{
    any::Any,
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, NUMBER_TYPES, STRING_TYPES, downcast_bool, downcast_number, downcast_string,
    },
    generated::types::{FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRString},
};
use haste_reflect::MetaValue;
use liquid::{Object, model::KString};
use liquid_core::Value;

/// Lazy `MetaValue` wrapper around a liquid `Object`.
///
/// Field values are converted from liquid to FHIR only on first access via
/// `get_field`. Field name strings are leaked once per instance to satisfy
/// the `&'static str` contract of `MetaValue::fields`.
#[derive(Debug)]
struct MetaValueLiquidObject {
    obj: Object,
    /// Converted field values keyed by field name. Entries are never removed,
    /// so the `Box` contents have stable heap addresses suitable for unsafe
    /// lifetime extension in `get_field`.
    cache: Mutex<HashMap<KString, Box<dyn MetaValue + Send + Sync>>>,
    static_fields: OnceLock<Vec<&'static str>>,
}

impl MetaValueLiquidObject {
    fn new(obj: Object) -> Self {
        Self {
            obj,
            cache: Mutex::new(HashMap::new()),
            static_fields: OnceLock::new(),
        }
    }
}

impl MetaValue for MetaValueLiquidObject {
    fn fields(&self) -> Vec<&'static str> {
        self.static_fields
            .get_or_init(|| {
                self.obj
                    .keys()
                    .map(|k| Box::leak(k.to_string().into_boxed_str()) as &'static str)
                    .collect()
            })
            .clone()
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        let key = self.obj.keys().find(|k| k.as_str() == field)?.clone();
        let mut cache = self.cache.lock().unwrap();
        if !cache.contains_key(&key) {
            let converted = liquid_to_metavalue(self.obj[&key].clone());
            cache.insert(key.clone(), Box::new(LiquidMetaCollection(converted)));
        }
        // SAFETY: `Box` contents have a stable heap address. The cache only
        // grows (entries are never removed or reallocated out from under the
        // pointer), so this reference is valid for the lifetime of `self` ('a).
        let ptr: *const dyn MetaValue = cache[&key].as_ref();
        drop(cache);
        Some(unsafe { &*ptr })
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn fhir_type(&self) -> &'static str {
        "BackboneElement"
    }

    fn is_many(&self) -> bool {
        false
    }
}

/// Wraps the `Vec` result of `liquid_to_fhir` for a single object field.
///
/// A single-element collection delegates field navigation to its inner value.
/// A multi-element collection exposes all elements via `flatten`.
#[derive(Debug)]
struct LiquidMetaCollection(Vec<Box<dyn MetaValue + Send + Sync>>);

impl MetaValue for LiquidMetaCollection {
    fn fields(&self) -> Vec<&'static str> {
        self.0.first().map(|v| v.fields()).unwrap_or_default()
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        if self.0.len() == 1 {
            self.0[0].get_field(field)
        } else {
            None
        }
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a dyn MetaValue> {
        self.0.get(index).map(|v| v.as_ref() as &dyn MetaValue)
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        self.0.iter().flat_map(|v| v.flatten()).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn fhir_type(&self) -> &'static str {
        self.0
            .first()
            .map(|v| v.fhir_type())
            .unwrap_or("BackboneElement")
    }

    fn is_many(&self) -> bool {
        self.0.len() != 1
    }
}

/// Convert a liquid `Value` to FHIR context entries.
///
/// Scalars map to a single FHIR primitive. Arrays are flattened so each
/// element becomes a separate context value, matching the FHIRPath collection
/// model. Objects become a lazy `BackboneElement`-typed `MetaObject`. Nil
/// produces an empty context.
pub fn liquid_to_metavalue(value: Value) -> Vec<Box<dyn MetaValue + Send + Sync>> {
    match value {
        Value::Scalar(s) => {
            // Bool must be checked before integer/float since liquid booleans
            // can round-trip through to_integer (true → 1).
            if let Some(b) = s.to_bool() {
                vec![Box::new(FHIRBoolean {
                    value: Some(b),
                    ..Default::default()
                })]
            } else if let Some(i) = s.to_integer() {
                vec![Box::new(FHIRInteger {
                    value: Some(i),
                    ..Default::default()
                })]
            } else if let Some(f) = s.to_float() {
                vec![Box::new(FHIRDecimal {
                    value: Some(f),
                    ..Default::default()
                })]
            } else {
                vec![Box::new(FHIRString {
                    value: Some(s.into_string().to_string()),
                    ..Default::default()
                })]
            }
        }
        Value::Array(arr) => arr.into_iter().flat_map(liquid_to_metavalue).collect(),
        Value::Object(obj) => vec![Box::new(MetaValueLiquidObject::new(obj))],
        _ => vec![],
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
            1 => fhir_to_liquid(items[0]),
            _ => Value::Array(items.into_iter().map(fhir_to_liquid).collect()),
        };
        obj.insert(KString::from_static(field), converted);
    }
    Value::Object(obj)
}
