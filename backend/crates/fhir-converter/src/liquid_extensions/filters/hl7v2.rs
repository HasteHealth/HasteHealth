use liquid_core::Expression;
use liquid_core::Result;
use liquid_core::Runtime;
use liquid_core::{
    Display_filter, Filter, FilterParameters, FilterReflection, FromFilterParameters, ParseFilter,
};
use liquid_core::{Value, ValueView};

#[derive(Debug, FilterParameters)]
struct HL7V2Args {
    #[parameter(description = "The HL7V2 Field accessor.", arg_type = "str")]
    accessor: Expression,
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "hl7v2",
    description = "Evaluates an HL7V2 field accessor.",
    parameters(HL7V2Args),
    parsed(HL7V2Filter)
)]
pub struct HL7V2;

#[derive(Debug, FromFilterParameters, Display_filter)]
#[name = "hl7v2"]
struct HL7V2Filter {
    #[parameters]
    args: HL7V2Args,
}

impl Filter for HL7V2Filter {
    fn evaluate(&self, _input: &dyn ValueView, _runtime: &dyn Runtime) -> Result<Value> {
        todo!();
    }
}
