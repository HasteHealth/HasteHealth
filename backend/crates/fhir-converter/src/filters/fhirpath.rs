use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, NUMBER_TYPES, STRING_TYPES, downcast_bool, downcast_number, downcast_string,
    },
    generated::types::{FHIRBoolean, FHIRDecimal, FHIRInteger, FHIRString},
};
use haste_reflect::MetaValue;
use liquid_core::Expression;
use liquid_core::Runtime;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Error, Result};
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

/// Convert a liquid `Value` scalar to the matching FHIR primitive type.
/// Returns `None` for Nil or complex (Array/Object) values.
fn liquid_to_fhir(value: Value) -> Option<Box<dyn MetaValue + Send + Sync>> {
    let scalar = match value {
        Value::Scalar(s) => s,
        _ => return None,
    };
    if let Some(b) = scalar.to_bool() {
        return Some(Box::new(FHIRBoolean {
            value: Some(b),
            ..Default::default()
        }));
    }
    if let Some(i) = scalar.to_integer() {
        return Some(Box::new(FHIRInteger {
            value: Some(i),
            ..Default::default()
        }));
    }
    if let Some(f) = scalar.to_float() {
        return Some(Box::new(FHIRDecimal {
            value: Some(f),
            ..Default::default()
        }));
    }
    let s = scalar.into_string().to_string();
    Some(Box::new(FHIRString {
        value: Some(s),
        ..Default::default()
    }))
}

/// Convert a `MetaValue` result to a liquid `Value` using `fhir_type()`.
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
    Value::scalar(format!("{:?}", value))
}

impl Filter for FHIRPathFilter {
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;
        let fhirpath = args.fhirpath;

        if fhirpath.is_empty() {
            return Err(Error::with_msg("FHIRPath expression cannot be empty"));
        }

        let fhir_input: Vec<Box<dyn MetaValue + Send + Sync>> = liquid_to_fhir(input.to_value())
            .map(|v| vec![v])
            .unwrap_or_default();

        let values = tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let refs: Vec<&dyn MetaValue> = fhir_input
                    .iter()
                    .map(|v| v.as_ref() as &dyn MetaValue)
                    .collect();

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
