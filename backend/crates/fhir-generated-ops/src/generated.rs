#![allow(non_snake_case)]
use oxidized_fhir_model::r4::generated::resources::*;
use oxidized_fhir_model::r4::generated::types::*;
use oxidized_fhir_operation_error::*;
use oxidized_fhir_ops::derive::{FromParameters, ToParameters};
pub mod ProjectInformation {
    use super::*;
    pub const CODE: &str = "current-project";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub project: Project,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ActivityDefinitionApply {
    use super::*;
    pub const CODE: &str = "apply";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub activityDefinition: Option<ActivityDefinition>,
        pub subject: Vec<FHIRString>,
        pub encounter: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub organization: Option<FHIRString>,
        pub userType: Option<CodeableConcept>,
        pub userLanguage: Option<CodeableConcept>,
        pub userTaskContext: Option<CodeableConcept>,
        pub setting: Option<CodeableConcept>,
        pub settingContext: Option<CodeableConcept>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            value.return_
        }
    }
}
pub mod ActivityDefinitionDataRequirements {
    use super::*;
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
pub mod CapabilityStatementConforms {
    use super::*;
    pub const CODE: &str = "conforms";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub left: Option<FHIRString>,
        pub right: Option<FHIRString>,
        pub mode: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub issues: OperationOutcome,
        pub union: Option<CapabilityStatement>,
        pub intersection: Option<CapabilityStatement>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CapabilityStatementImplements {
    use super::*;
    pub const CODE: &str = "implements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub server: Option<FHIRString>,
        pub client: Option<FHIRString>,
        pub resource: Option<CapabilityStatement>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
pub mod CapabilityStatementSubset {
    use super::*;
    pub const CODE: &str = "subset";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub server: Option<FHIRUri>,
        pub resource: Vec<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: CapabilityStatement,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::CapabilityStatement(value.return_)
        }
    }
}
pub mod CapabilityStatementVersions {
    use super::*;
    pub const CODE: &str = "versions";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub version: Vec<FHIRCode>,
        pub default: FHIRCode,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ChargeItemDefinitionApply {
    use super::*;
    pub const CODE: &str = "apply";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub chargeItem: Reference,
        pub account: Option<Reference>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            value.return_
        }
    }
}
pub mod ClaimSubmit {
    use super::*;
    pub const CODE: &str = "submit";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub resource: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CodeSystemFindMatches {
    use super::*;
    pub const CODE: &str = "find-matches";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct InputPropertySubproperty {
        pub code: FHIRCode,
        pub value: ParametersParameterValueTypeChoice,
    }
    impl From<InputPropertySubproperty> for Resource {
        fn from(value: InputPropertySubproperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct InputProperty {
        pub code: FHIRCode,
        pub value: Option<ParametersParameterValueTypeChoice>,
        #[parameter_nested]
        pub subproperty: Option<Vec<InputPropertySubproperty>>,
    }
    impl From<InputProperty> for Resource {
        fn from(value: InputProperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        #[parameter_nested]
        pub property: Option<Vec<InputProperty>>,
        pub exact: FHIRBoolean,
        pub compositional: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatchUnmatchedProperty {
        pub code: FHIRCode,
        pub value: ParametersParameterValueTypeChoice,
    }
    impl From<OutputMatchUnmatchedProperty> for Resource {
        fn from(value: OutputMatchUnmatchedProperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatchUnmatched {
        pub code: FHIRCode,
        pub value: ParametersParameterValueTypeChoice,
        #[parameter_nested]
        pub property: Option<Vec<OutputMatchUnmatchedProperty>>,
    }
    impl From<OutputMatchUnmatched> for Resource {
        fn from(value: OutputMatchUnmatched) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatch {
        pub code: Coding,
        #[parameter_nested]
        pub unmatched: Option<Vec<OutputMatchUnmatched>>,
        pub comment: Option<FHIRString>,
    }
    impl From<OutputMatch> for Resource {
        fn from(value: OutputMatch) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "match"]
        #[parameter_nested]
        pub match_: Option<Vec<OutputMatch>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CodeSystemLookup {
    use super::*;
    pub const CODE: &str = "lookup";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub code: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub coding: Option<Coding>,
        pub date: Option<FHIRDateTime>,
        pub displayLanguage: Option<FHIRCode>,
        pub property: Option<Vec<FHIRCode>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputDesignation {
        pub language: Option<FHIRCode>,
        #[parameter_rename = "use"]
        pub use_: Option<Coding>,
        pub value: FHIRString,
    }
    impl From<OutputDesignation> for Resource {
        fn from(value: OutputDesignation) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputPropertySubproperty {
        pub code: FHIRCode,
        pub value: ParametersParameterValueTypeChoice,
        pub description: Option<FHIRString>,
    }
    impl From<OutputPropertySubproperty> for Resource {
        fn from(value: OutputPropertySubproperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputProperty {
        pub code: FHIRCode,
        pub value: Option<ParametersParameterValueTypeChoice>,
        pub description: Option<FHIRString>,
        #[parameter_nested]
        pub subproperty: Option<Vec<OutputPropertySubproperty>>,
    }
    impl From<OutputProperty> for Resource {
        fn from(value: OutputProperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub name: FHIRString,
        pub version: Option<FHIRString>,
        pub display: FHIRString,
        #[parameter_nested]
        pub designation: Option<Vec<OutputDesignation>>,
        #[parameter_nested]
        pub property: Option<Vec<OutputProperty>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CodeSystemSubsumes {
    use super::*;
    pub const CODE: &str = "subsumes";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub codeA: Option<FHIRCode>,
        pub codeB: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub codingA: Option<Coding>,
        pub codingB: Option<Coding>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub outcome: FHIRCode,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CodeSystemValidateCode {
    use super::*;
    pub const CODE: &str = "validate-code";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub url: Option<FHIRUri>,
        pub codeSystem: Option<CodeSystem>,
        pub code: Option<FHIRCode>,
        pub version: Option<FHIRString>,
        pub display: Option<FHIRString>,
        pub coding: Option<Coding>,
        pub codeableConcept: Option<CodeableConcept>,
        pub date: Option<FHIRDateTime>,
        #[parameter_rename = "abstract"]
        pub abstract_: Option<FHIRBoolean>,
        pub displayLanguage: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub display: Option<FHIRString>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CompositionDocument {
    use super::*;
    pub const CODE: &str = "document";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub id: Option<FHIRUri>,
        pub persist: Option<FHIRBoolean>,
        pub graph: Option<FHIRUri>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ConceptMapClosure {
    use super::*;
    pub const CODE: &str = "closure";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub name: FHIRString,
        pub concept: Option<Vec<Coding>>,
        pub version: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: ConceptMap,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::ConceptMap(value.return_)
        }
    }
}
pub mod ConceptMapTranslate {
    use super::*;
    pub const CODE: &str = "translate";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct InputDependency {
        pub element: Option<FHIRUri>,
        pub concept: Option<CodeableConcept>,
    }
    impl From<InputDependency> for Resource {
        fn from(value: InputDependency) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub url: Option<FHIRUri>,
        pub conceptMap: Option<ConceptMap>,
        pub conceptMapVersion: Option<FHIRString>,
        pub code: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub source: Option<FHIRUri>,
        pub coding: Option<Coding>,
        pub codeableConcept: Option<CodeableConcept>,
        pub target: Option<FHIRUri>,
        pub targetsystem: Option<FHIRUri>,
        #[parameter_nested]
        pub dependency: Option<Vec<InputDependency>>,
        pub reverse: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatchProduct {
        pub element: Option<FHIRUri>,
        pub concept: Option<Coding>,
    }
    impl From<OutputMatchProduct> for Resource {
        fn from(value: OutputMatchProduct) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatch {
        pub equivalence: Option<FHIRCode>,
        pub concept: Option<Coding>,
        #[parameter_nested]
        pub product: Option<Vec<OutputMatchProduct>>,
        pub source: Option<FHIRUri>,
    }
    impl From<OutputMatch> for Resource {
        fn from(value: OutputMatch) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        #[parameter_rename = "match"]
        #[parameter_nested]
        pub match_: Option<Vec<OutputMatch>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod CoverageEligibilityRequestSubmit {
    use super::*;
    pub const CODE: &str = "submit";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub resource: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod EncounterEverything {
    use super::*;
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod GroupEverything {
    use super::*;
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub start: Option<FHIRDate>,
        pub end: Option<FHIRDate>,
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod LibraryDataRequirements {
    use super::*;
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub target: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
pub mod ListFind {
    use super::*;
    pub const CODE: &str = "find";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub patient: FHIRId,
        pub name: FHIRCode,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod MeasureCareGaps {
    use super::*;
    pub const CODE: &str = "care-gaps";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub topic: FHIRString,
        pub subject: FHIRString,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod MeasureCollectData {
    use super::*;
    pub const CODE: &str = "collect-data";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub measure: Option<FHIRString>,
        pub subject: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub measureReport: MeasureReport,
        pub resource: Option<Vec<Resource>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod MeasureDataRequirements {
    use super::*;
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
pub mod MeasureEvaluateMeasure {
    use super::*;
    pub const CODE: &str = "evaluate-measure";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub measure: Option<FHIRString>,
        pub reportType: Option<FHIRCode>,
        pub subject: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: MeasureReport,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::MeasureReport(value.return_)
        }
    }
}
pub mod MeasureSubmitData {
    use super::*;
    pub const CODE: &str = "submit-data";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub measureReport: MeasureReport,
        pub resource: Option<Vec<Resource>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod MedicinalProductEverything {
    use super::*;
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub _since: Option<FHIRInstant>,
        pub _count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod MessageHeaderProcessMessage {
    use super::*;
    pub const CODE: &str = "process-message";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub content: Bundle,
        #[parameter_rename = "async"]
        pub async_: Option<FHIRBoolean>,
        pub response_url: Option<FHIRUrl>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Option<Bundle>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_.unwrap_or_default())
        }
    }
}
pub mod NamingSystemPreferredId {
    use super::*;
    pub const CODE: &str = "preferred-id";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub id: FHIRString,
        #[parameter_rename = "type"]
        pub type_: FHIRCode,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub result: FHIRString,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ObservationLastn {
    use super::*;
    pub const CODE: &str = "lastn";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub max: Option<FHIRPositiveInt>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod ObservationStats {
    use super::*;
    pub const CODE: &str = "stats";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub subject: FHIRUri,
        pub code: Option<Vec<FHIRString>>,
        pub system: Option<FHIRUri>,
        pub coding: Option<Vec<Coding>>,
        pub duration: Option<FHIRDecimal>,
        pub period: Option<Period>,
        pub statistic: Vec<FHIRCode>,
        pub include: Option<FHIRBoolean>,
        pub limit: Option<FHIRPositiveInt>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub statistics: Vec<Observation>,
        pub source: Option<Vec<Observation>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod PatientEverything {
    use super::*;
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub start: Option<FHIRDate>,
        pub end: Option<FHIRDate>,
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod PatientMatch {
    use super::*;
    pub const CODE: &str = "match";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub resource: Resource,
        pub onlyCertainMatches: Option<FHIRBoolean>,
        pub count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
pub mod PlanDefinitionApply {
    use super::*;
    pub const CODE: &str = "apply";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub planDefinition: Option<PlanDefinition>,
        pub subject: Vec<FHIRString>,
        pub encounter: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub organization: Option<FHIRString>,
        pub userType: Option<CodeableConcept>,
        pub userLanguage: Option<CodeableConcept>,
        pub userTaskContext: Option<CodeableConcept>,
        pub setting: Option<CodeableConcept>,
        pub settingContext: Option<CodeableConcept>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: CarePlan,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::CarePlan(value.return_)
        }
    }
}
pub mod PlanDefinitionDataRequirements {
    use super::*;
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
pub mod ResourceConvert {
    use super::*;
    pub const CODE: &str = "convert";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub input: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub output: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ResourceGraph {
    use super::*;
    pub const CODE: &str = "graph";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub graph: FHIRUri,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub result: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ResourceGraphql {
    use super::*;
    pub const CODE: &str = "graphql";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub query: FHIRString,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub result: Binary,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ResourceMeta {
    use super::*;
    pub const CODE: &str = "meta";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Meta,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ResourceMetaAdd {
    use super::*;
    pub const CODE: &str = "meta-add";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub meta: Meta,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Meta,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ResourceMetaDelete {
    use super::*;
    pub const CODE: &str = "meta-delete";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub meta: Meta,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Meta,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ResourceValidate {
    use super::*;
    pub const CODE: &str = "validate";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub resource: Option<Resource>,
        pub mode: Option<FHIRCode>,
        pub profile: Option<FHIRUri>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
pub mod StructureDefinitionQuestionnaire {
    use super::*;
    pub const CODE: &str = "questionnaire";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[parameter_rename = "identifier"]
        pub identifier_: Option<FHIRString>,
        pub profile: Option<FHIRString>,
        pub url: Option<FHIRString>,
        pub supportedOnly: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Questionnaire,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Questionnaire(value.return_)
        }
    }
}
pub mod StructureDefinitionSnapshot {
    use super::*;
    pub const CODE: &str = "snapshot";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub definition: Option<StructureDefinition>,
        pub url: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: StructureDefinition,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::StructureDefinition(value.return_)
        }
    }
}
pub mod StructureMapTransform {
    use super::*;
    pub const CODE: &str = "transform";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub source: Option<FHIRUri>,
        pub content: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
pub mod ValueSetExpand {
    use super::*;
    pub const CODE: &str = "expand";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub url: Option<FHIRUri>,
        pub valueSet: Option<ValueSet>,
        pub valueSetVersion: Option<FHIRString>,
        pub context: Option<FHIRUri>,
        pub contextDirection: Option<FHIRCode>,
        pub filter: Option<FHIRString>,
        pub date: Option<FHIRDateTime>,
        pub offset: Option<FHIRInteger>,
        pub count: Option<FHIRInteger>,
        pub includeDesignations: Option<FHIRBoolean>,
        pub designation: Option<Vec<FHIRString>>,
        pub includeDefinition: Option<FHIRBoolean>,
        pub activeOnly: Option<FHIRBoolean>,
        pub excludeNested: Option<FHIRBoolean>,
        pub excludeNotForUI: Option<FHIRBoolean>,
        pub excludePostCoordinated: Option<FHIRBoolean>,
        pub displayLanguage: Option<FHIRCode>,
        pub exclude_system: Option<Vec<FHIRString>>,
        pub system_version: Option<Vec<FHIRString>>,
        pub check_system_version: Option<Vec<FHIRString>>,
        pub force_system_version: Option<Vec<FHIRString>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[parameter_rename = "return"]
        pub return_: ValueSet,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::ValueSet(value.return_)
        }
    }
}
pub mod ValueSetValidateCode {
    use super::*;
    pub const CODE: &str = "validate-code";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        pub url: Option<FHIRUri>,
        pub context: Option<FHIRUri>,
        pub valueSet: Option<ValueSet>,
        pub valueSetVersion: Option<FHIRString>,
        pub code: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub systemVersion: Option<FHIRString>,
        pub display: Option<FHIRString>,
        pub coding: Option<Coding>,
        pub codeableConcept: Option<CodeableConcept>,
        pub date: Option<FHIRDateTime>,
        #[parameter_rename = "abstract"]
        pub abstract_: Option<FHIRBoolean>,
        pub displayLanguage: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub display: Option<FHIRString>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
