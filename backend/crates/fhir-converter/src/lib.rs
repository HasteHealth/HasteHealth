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

impl Filter for FHIRPathFilter {
    fn evaluate(&self, _input: &dyn ValueView, runtime: &dyn Runtime) -> Result<Value> {
        let args = self.args.evaluate(runtime)?;

        let fhirpath = args.fhirpath;

        // let view_value = input.to_value();

        if !fhirpath.is_empty() {
            let result = tokio::task::block_in_place(|| {
                Handle::current().block_on(async {
                    haste_fhirpath::FPEngine::new()
                        .evaluate(fhirpath.as_str(), vec![])
                        .await
                })
            })
            .map_err(|err| Error::with_msg(format!("FHIRPath evaluation error: {}", err)))?;

            let k = result
                .iter()
                .map(|v| Value::scalar(format!("{:?}", v)))
                .collect::<Vec<_>>();

            Ok(k.into())
        } else {
            Err(Error::with_msg("FHIRPath expression cannot be empty"))
        }
    }
}

pub fn fhir_converter() {
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(FHIRPath)
        .build()
        .unwrap()
        .parse("Liquid! {{num | fhirpath: '5.5 / 2'}}")
        .unwrap();

    let globals = liquid::object!({
        "num": 4f64
    });

    let output = template.render(&globals).unwrap();

    println!("{}", output);
}
