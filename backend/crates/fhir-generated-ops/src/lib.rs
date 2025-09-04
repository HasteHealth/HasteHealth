#![allow(non_snake_case)]
use oxidized_fhir_model::r4::types::*;
use oxidized_fhir_ops::derive::ParametersParse;
pub mod ActivityDefinitionApply {
    use super::*;
    #[derive(ParametersParse)]
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
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod ActivityDefinitionDataRequirements {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {}
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Library,
    }
}
pub mod CapabilityStatementConforms {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub left: Option<FHIRString>,
        pub right: Option<FHIRString>,
        pub mode: Option<FHIRCode>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub issues: OperationOutcome,
        pub union: Option<CapabilityStatement>,
        pub intersection: Option<CapabilityStatement>,
    }
}
pub mod CapabilityStatementImplements {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub server: Option<FHIRString>,
        pub client: Option<FHIRString>,
        pub resource: Option<CapabilityStatement>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: OperationOutcome,
    }
}
pub mod CapabilityStatementSubset {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub server: Option<FHIRUri>,
        pub resource: Vec<FHIRCode>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: CapabilityStatement,
    }
}
pub mod CapabilityStatementVersions {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {}
    #[derive(ParametersParse)]
    pub struct Output {
        pub version: Vec<FHIRCode>,
        pub default: FHIRCode,
    }
}
pub mod ChargeItemDefinitionApply {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub chargeItem: Reference,
        pub account: Option<Reference>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod ClaimSubmit {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub resource: Resource,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod CodeSystemFindMatches {
    use super::*;
    #[derive(ParametersParse)]
    pub struct InputPropertySubproperty {
        pub code: FHIRCode,
        pub value: Element,
    }
    #[derive(ParametersParse)]
    pub struct InputProperty {
        pub code: FHIRCode,
        pub value: Option<Element>,
        pub subproperty: Option<Vec<InputPropertySubproperty>>,
    }
    #[derive(ParametersParse)]
    pub struct Input {
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub property: Option<Vec<InputProperty>>,
        pub exact: FHIRBoolean,
        pub compositional: Option<FHIRBoolean>,
    }
    #[derive(ParametersParse)]
    pub struct OutputMatchUnmatchedProperty {
        pub code: FHIRCode,
        pub value: Element,
    }
    #[derive(ParametersParse)]
    pub struct OutputMatchUnmatched {
        pub code: FHIRCode,
        pub value: Element,
        pub property: Option<Vec<OutputMatchUnmatchedProperty>>,
    }
    #[derive(ParametersParse)]
    pub struct OutputMatch {
        pub code: Coding,
        pub unmatched: Option<Vec<OutputMatchUnmatched>>,
        pub comment: Option<FHIRString>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub match_: Option<Vec<OutputMatch>>,
    }
}
pub mod CodeSystemLookup {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub code: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub coding: Option<Coding>,
        pub date: Option<FHIRDateTime>,
        pub displayLanguage: Option<FHIRCode>,
        pub property: Option<Vec<FHIRCode>>,
    }
    #[derive(ParametersParse)]
    pub struct OutputDesignation {
        pub language: Option<FHIRCode>,
        pub use_: Option<Coding>,
        pub value: FHIRString,
    }
    #[derive(ParametersParse)]
    pub struct OutputPropertySubproperty {
        pub code: FHIRCode,
        pub value: Element,
        pub description: Option<FHIRString>,
    }
    #[derive(ParametersParse)]
    pub struct OutputProperty {
        pub code: FHIRCode,
        pub value: Option<Element>,
        pub description: Option<FHIRString>,
        pub subproperty: Option<Vec<OutputPropertySubproperty>>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub name: FHIRString,
        pub version: Option<FHIRString>,
        pub display: FHIRString,
        pub designation: Option<Vec<OutputDesignation>>,
        pub property: Option<Vec<OutputProperty>>,
    }
}
pub mod CodeSystemSubsumes {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub codeA: Option<FHIRCode>,
        pub codeB: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub codingA: Option<Coding>,
        pub codingB: Option<Coding>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub outcome: FHIRCode,
    }
}
pub mod CodeSystemValidateCode {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub url: Option<FHIRUri>,
        pub codeSystem: Option<CodeSystem>,
        pub code: Option<FHIRCode>,
        pub version: Option<FHIRString>,
        pub display: Option<FHIRString>,
        pub coding: Option<Coding>,
        pub codeableConcept: Option<CodeableConcept>,
        pub date: Option<FHIRDateTime>,
        pub abstract_: Option<FHIRBoolean>,
        pub displayLanguage: Option<FHIRCode>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub display: Option<FHIRString>,
    }
}
pub mod CompositionDocument {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub id: Option<FHIRUri>,
        pub persist: Option<FHIRBoolean>,
        pub graph: Option<FHIRUri>,
    }
    #[derive(ParametersParse)]
    pub struct Output {}
}
pub mod ConceptMapClosure {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub name: FHIRString,
        pub concept: Option<Vec<Coding>>,
        pub version: Option<FHIRString>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: ConceptMap,
    }
}
pub mod ConceptMapTranslate {
    use super::*;
    #[derive(ParametersParse)]
    pub struct InputDependency {
        pub element: Option<FHIRUri>,
        pub concept: Option<CodeableConcept>,
    }
    #[derive(ParametersParse)]
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
        pub dependency: Option<Vec<InputDependency>>,
        pub reverse: Option<FHIRBoolean>,
    }
    #[derive(ParametersParse)]
    pub struct OutputMatchProduct {
        pub element: Option<FHIRUri>,
        pub concept: Option<Coding>,
    }
    #[derive(ParametersParse)]
    pub struct OutputMatch {
        pub equivalence: Option<FHIRCode>,
        pub concept: Option<Coding>,
        pub product: Option<Vec<OutputMatchProduct>>,
        pub source: Option<FHIRUri>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub match_: Option<Vec<OutputMatch>>,
    }
}
pub mod CoverageEligibilityRequestSubmit {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub resource: Resource,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod EncounterEverything {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod GroupEverything {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub start: Option<FHIRDate>,
        pub end: Option<FHIRDate>,
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod LibraryDataRequirements {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub target: Option<FHIRString>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Library,
    }
}
pub mod ListFind {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub patient: FHIRId,
        pub name: FHIRCode,
    }
    #[derive(ParametersParse)]
    pub struct Output {}
}
pub mod MeasureCareGaps {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub topic: FHIRString,
        pub subject: FHIRString,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod MeasureCollectData {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub measure: Option<FHIRString>,
        pub subject: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub measureReport: MeasureReport,
        pub resource: Option<Vec<Resource>>,
    }
}
pub mod MeasureDataRequirements {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Library,
    }
}
pub mod MeasureEvaluateMeasure {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub measure: Option<FHIRString>,
        pub reportType: Option<FHIRCode>,
        pub subject: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: MeasureReport,
    }
}
pub mod MeasureSubmitData {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub measureReport: MeasureReport,
        pub resource: Option<Vec<Resource>>,
    }
    #[derive(ParametersParse)]
    pub struct Output {}
}
pub mod MedicinalProductEverything {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub _since: Option<FHIRInstant>,
        pub _count: Option<FHIRInteger>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod MessageHeaderProcessMessage {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub content: Bundle,
        pub async_: Option<FHIRBoolean>,
        pub response_url: Option<FHIRUrl>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Option<Bundle>,
    }
}
pub mod NamingSystemPreferredId {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub id: FHIRString,
        pub type_: FHIRCode,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub result: FHIRString,
    }
}
pub mod ObservationLastn {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub max: Option<FHIRPositiveInt>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod ObservationStats {
    use super::*;
    #[derive(ParametersParse)]
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
    #[derive(ParametersParse)]
    pub struct Output {
        pub statistics: Vec<Observation>,
        pub source: Option<Vec<Observation>>,
    }
}
pub mod PatientEverything {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub start: Option<FHIRDate>,
        pub end: Option<FHIRDate>,
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod PatientMatch {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub resource: Resource,
        pub onlyCertainMatches: Option<FHIRBoolean>,
        pub count: Option<FHIRInteger>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod PlanDefinitionApply {
    use super::*;
    #[derive(ParametersParse)]
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
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: CarePlan,
    }
}
pub mod PlanDefinitionDataRequirements {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {}
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Library,
    }
}
pub mod ResourceConvert {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub input: Resource,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub output: Resource,
    }
}
pub mod ResourceGraph {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub graph: FHIRUri,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub result: Bundle,
    }
}
pub mod ResourceGraphql {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub query: FHIRString,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub result: Binary,
    }
}
pub mod ResourceMeta {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {}
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Meta,
    }
}
pub mod ResourceMetaAdd {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub meta: Meta,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Meta,
    }
}
pub mod ResourceMetaDelete {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub meta: Meta,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Meta,
    }
}
pub mod ResourceValidate {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub resource: Option<Resource>,
        pub mode: Option<FHIRCode>,
        pub profile: Option<FHIRUri>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: OperationOutcome,
    }
}
pub mod StructureDefinitionQuestionnaire {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub identifier_: Option<FHIRString>,
        pub profile: Option<FHIRString>,
        pub url: Option<FHIRString>,
        pub supportedOnly: Option<FHIRBoolean>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Questionnaire,
    }
}
pub mod StructureDefinitionSnapshot {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub definition: Option<StructureDefinition>,
        pub url: Option<FHIRString>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: StructureDefinition,
    }
}
pub mod StructureMapTransform {
    use super::*;
    #[derive(ParametersParse)]
    pub struct Input {
        pub source: Option<FHIRUri>,
        pub content: Resource,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod ValueSetExpand {
    use super::*;
    #[derive(ParametersParse)]
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
    #[derive(ParametersParse)]
    pub struct Output {
        pub return_: ValueSet,
    }
}
pub mod ValueSetValidateCode {
    use super::*;
    #[derive(ParametersParse)]
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
        pub abstract_: Option<FHIRBoolean>,
        pub displayLanguage: Option<FHIRCode>,
    }
    #[derive(ParametersParse)]
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub display: Option<FHIRString>,
    }
}
