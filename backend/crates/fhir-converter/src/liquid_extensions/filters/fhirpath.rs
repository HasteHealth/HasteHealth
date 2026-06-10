use haste_reflect::MetaValue;
use liquid_core::Expression;
use liquid_core::Runtime;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Error, Result};
use liquid_core::{Value, ValueView};
use tokio::runtime::Handle;

use crate::liquid_extensions::conversions::{fhir_to_liquid, liquid_to_metavalue};

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

impl Filter for FHIRPathFilter {
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;
        let fhirpath = args.fhirpath;

        if fhirpath.is_empty() {
            return Err(Error::with_msg("FHIRPath expression cannot be empty"));
        }

        let owned = liquid_to_metavalue(input.to_value())?;

        let values = tokio::task::block_in_place(|| {
            Handle::current().block_on(async {
                let refs: Vec<&dyn MetaValue> =
                    owned.iter().map(|b| b.as_ref() as &dyn MetaValue).collect();
                let ctx = haste_fhirpath::FPEngine::new()
                    .evaluate(fhirpath.as_str(), refs)
                    .await?;

                let converted: Vec<Value> = ctx.iter().map(|value| fhir_to_liquid(value)).collect();
                Ok::<Vec<Value>, haste_fhirpath::FHIRPathError>(converted)
            })
        })
        .map_err(|err| Error::with_msg(format!("FHIRPath evaluation error: {}", err)))?;

        Ok(Value::Array(values))
    }
}
