use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, NUMBER_TYPES, STRING_TYPES, downcast_bool, downcast_number, downcast_string,
    },
    generated::types::{FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRString},
};
use haste_reflect::MetaValue;
use liquid_core::Expression;
use liquid_core::Runtime;
use liquid_core::model::KString;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Error, Object, Result};
use liquid_core::{Value, ValueView};
use tokio::runtime::Handle;

#[derive(Debug, FilterParameters)]
struct FHIRPathArgs {
    #[parameter(description = "The FHIRPath expression to evaluate.", arg_type = "str")]
    fhirpath: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "fhirpath",
    description = "Evaluates a FHIRPath expression.",
    parameters(FHIRPathArgs),
    parsed(FHIRPathFilter)
)]
pub struct FHIRPath;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "fhirpath"]
struct FHIRPathFilter {
    #[parameters]
    args: FHIRPathArgs,
}

/// Convert a liquid `Value` to FHIR context entries.
///
/// Scalars map to a single FHIR primitive. Arrays are flattened so each
/// element becomes a separate context value, matching the FHIRPath collection
/// model. Objects and Nil produce an empty context.
fn liquid_to_fhir(value: Value) -> Vec<Box<dyn MetaValue + Send + Sync>> {
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
        Value::Array(arr) => arr.into_iter().flat_map(liquid_to_fhir).collect(),
        Value::Object(_) => vec![],
        _ => vec![],
    }
}

/// Convert a `MetaValue` to a liquid `Value`.
///
/// Primitive FHIR types are dispatched via `fhir_type()` and converted to
/// the matching liquid scalar. Complex types are recursively converted to
/// `Value::Object` using the same `flatten()` traversal that the FHIRPath
/// engine uses, so field semantics are consistent.
fn fhir_to_liquid(value: &dyn MetaValue) -> Value {
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

impl Filter for FHIRPathFilter {
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;
        let fhirpath = args.fhirpath;

        if fhirpath.is_empty() {
            return Err(Error::with_msg("FHIRPath expression cannot be empty"));
        }

        let owned = liquid_to_fhir(input.to_value());

        let values = tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let refs: Vec<&dyn MetaValue> =
                    owned.iter().map(|b| b.as_ref() as &dyn MetaValue).collect();
                let ctx = haste_fhirpath::FPEngine::new()
                    .evaluate(fhirpath.as_str(), refs)
                    .await?;

                let converted: Vec<Value> = ctx.iter().map(fhir_to_liquid).collect();
                Ok::<Vec<Value>, haste_fhirpath::FHIRPathError>(converted)
            })
        })
        .map_err(|err| Error::with_msg(format!("FHIRPath evaluation error: {}", err)))?;

        match values.len() {
            0 => Ok(Value::Nil),
            1 => Ok(values.into_iter().next().unwrap()),
            _ => Ok(Value::Array(values)),
        }
    }
}
