pub mod generated;

#[cfg(test)]
mod tests {
    use oxidized_fhir_model::r4::generated::{
        resources::{Patient, Resource},
        types::FHIRString,
    };

    use super::*;

    #[test]
    fn from_parameter() {
        let result = generated::ActivityDefinitionApply::Output::try_from(vec![
            oxidized_fhir_model::r4::generated::resources::ParametersParameter {
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

        assert_eq!(result.is_ok(), true);
    }
}
