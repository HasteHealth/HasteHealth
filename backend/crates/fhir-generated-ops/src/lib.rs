pub mod generated;

#[cfg(test)]
mod tests {
    use oxidized_fhir_model::r4::generated::{
        resources::{Parameters, Patient, Resource},
        types::{CodeableConcept, Coding, FHIRString, FHIRUri},
    };

    use super::*;

    #[test]
    fn from_parameter() {
        let result = generated::ActivityDefinitionApply::Output::try_from(vec![
            oxidized_fhir_model::r4::generated::resources::ParametersParameter {
                name: Box::new(FHIRString {
                    value: Some("return".to_string()),
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
    #[test]
    fn to_parameter() {
        let input = generated::ActivityDefinitionApply::Input {
            activityDefinition: None,
            subject: vec![],
            encounter: Some(FHIRString {
                value: Some("encounter".to_string()),
                ..Default::default()
            }),
            practitioner: Some(FHIRString {
                value: Some("encounter".to_string()),
                ..Default::default()
            }),
            organization: Some(FHIRString {
                value: Some("Patient".to_string()),
                ..Default::default()
            }),
            userType: Some(CodeableConcept {
                coding: Some(vec![Box::new(Coding {
                    system: Some(Box::new(FHIRUri {
                        value: Some(
                            "http://terminology.hl7.org/CodeSystem/encounter-type".to_string(),
                        ),
                        ..Default::default()
                    })),
                    ..Default::default()
                })]),
                ..Default::default()
            }),
            userLanguage: None,
            userTaskContext: None,
            setting: None,
            settingContext: None,
        };

        let param = Parameters {
            parameter: Some(input.into()),
            ..Default::default()
        };

        assert_eq!(param.parameter.as_ref().unwrap().len(), 4);
    }
}
