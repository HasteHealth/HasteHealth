pub mod generated;

#[cfg(test)]
mod tests {
    use oxidized_fhir_model::r4::types::{FHIRString, Patient, Resource};

    use super::*;

    #[test]
    fn from_parameter() {
        let result = generated::ActivityDefinitionApply::Output::try_from(vec![
            oxidized_fhir_model::r4::types::ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some("return_".to_string()),
                    ..Default::default()
                }),
                resource: Some(Box::new(Resource::Patient(Patient {
                    ..Default::default()
                }))),
                ..Default::default()
            },
        ]);

        println!("Result: {:?}", result);

        assert_eq!(result.is_ok(), true);
    }
}
