#![allow(non_snake_case)]
use oxidized_fhir_model::r4::types::*;
pub mod ActivityDefinitionApply {
    use super::*;
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
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod ActivityDefinitionDataRequirements {
    use super::*;
    pub struct Input {}
    pub struct Output {
        pub return_: Library,
    }
}
pub mod CapabilityStatementConforms {
    use super::*;
    pub struct Input {
        pub left: Option<FHIRString>,
        pub right: Option<FHIRString>,
        pub mode: Option<FHIRCode>,
    }
    pub struct Output {
        pub issues: OperationOutcome,
        pub union: Option<CapabilityStatement>,
        pub intersection: Option<CapabilityStatement>,
    }
}
pub mod CapabilityStatementImplements {
    use super::*;
    pub struct Input {
        pub server: Option<FHIRString>,
        pub client: Option<FHIRString>,
        pub resource: Option<CapabilityStatement>,
    }
    pub struct Output {
        pub return_: OperationOutcome,
    }
}
pub mod CapabilityStatementSubset {
    use super::*;
    pub struct Input {
        pub server: Option<FHIRUri>,
        pub resource: Vec<FHIRCode>,
    }
    pub struct Output {
        pub return_: CapabilityStatement,
    }
}
pub mod CapabilityStatementVersions {
    use super::*;
    pub struct Input {}
    pub struct Output {
        pub version: Vec<FHIRCode>,
        pub default: FHIRCode,
    }
}
pub mod ChargeItemDefinitionApply {
    use super::*;
    pub struct Input {
        pub chargeItem: Reference,
        pub account: Option<Reference>,
    }
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod ClaimSubmit {
    use super::*;
    pub struct Input {
        pub resource: Resource,
    }
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod CodeSystemFindMatches {
    use super::*;
    pub struct InputPropertySubproperty {
        pub code: FHIRCode,
        pub value: Element,
    }
    pub struct InputProperty {
        pub code: FHIRCode,
        pub value: Option<Element>,
        pub subproperty: Option<Vec<InputPropertySubproperty>>,
    }
    pub struct Input {
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub property: Option<Vec<InputProperty>>,
        pub exact: FHIRBoolean,
        pub compositional: Option<FHIRBoolean>,
    }
    pub struct OutputMatchUnmatchedProperty {
        pub code: FHIRCode,
        pub value: Element,
    }
    pub struct OutputMatchUnmatched {
        pub code: FHIRCode,
        pub value: Element,
        pub property: Option<Vec<OutputMatchUnmatchedProperty>>,
    }
    pub struct OutputMatch {
        pub code: Coding,
        pub unmatched: Option<Vec<OutputMatchUnmatched>>,
        pub comment: Option<FHIRString>,
    }
    pub struct Output {
        pub match_: Option<Vec<OutputMatch>>,
    }
}
pub mod CodeSystemLookup {
    use super::*;
    pub struct Input {
        pub code: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub coding: Option<Coding>,
        pub date: Option<FHIRDateTime>,
        pub displayLanguage: Option<FHIRCode>,
        pub property: Option<Vec<FHIRCode>>,
    }
    pub struct OutputDesignation {
        pub language: Option<FHIRCode>,
        pub use_: Option<Coding>,
        pub value: FHIRString,
    }
    pub struct OutputPropertySubproperty {
        pub code: FHIRCode,
        pub value: Element,
        pub description: Option<FHIRString>,
    }
    pub struct OutputProperty {
        pub code: FHIRCode,
        pub value: Option<Element>,
        pub description: Option<FHIRString>,
        pub subproperty: Option<Vec<OutputPropertySubproperty>>,
    }
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
    pub struct Input {
        pub codeA: Option<FHIRCode>,
        pub codeB: Option<FHIRCode>,
        pub system: Option<FHIRUri>,
        pub version: Option<FHIRString>,
        pub codingA: Option<Coding>,
        pub codingB: Option<Coding>,
    }
    pub struct Output {
        pub outcome: FHIRCode,
    }
}
pub mod CodeSystemValidateCode {
    use super::*;
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
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub display: Option<FHIRString>,
    }
}
pub mod CompositionDocument {
    use super::*;
    pub struct Input {
        pub id: Option<FHIRUri>,
        pub persist: Option<FHIRBoolean>,
        pub graph: Option<FHIRUri>,
    }
    pub struct Output {}
}
pub mod ConceptMapClosure {
    use super::*;
    pub struct Input {
        pub name: FHIRString,
        pub concept: Option<Vec<Coding>>,
        pub version: Option<FHIRString>,
    }
    pub struct Output {
        pub return_: ConceptMap,
    }
}
pub mod ConceptMapTranslate {
    use super::*;
    pub struct InputDependency {
        pub element: Option<FHIRUri>,
        pub concept: Option<CodeableConcept>,
    }
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
    pub struct OutputMatchProduct {
        pub element: Option<FHIRUri>,
        pub concept: Option<Coding>,
    }
    pub struct OutputMatch {
        pub equivalence: Option<FHIRCode>,
        pub concept: Option<Coding>,
        pub product: Option<Vec<OutputMatchProduct>>,
        pub source: Option<FHIRUri>,
    }
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub match_: Option<Vec<OutputMatch>>,
    }
}
pub mod CoverageEligibilityRequestSubmit {
    use super::*;
    pub struct Input {
        pub resource: Resource,
    }
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod EncounterEverything {
    use super::*;
    pub struct Input {
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod GroupEverything {
    use super::*;
    pub struct Input {
        pub start: Option<FHIRDate>,
        pub end: Option<FHIRDate>,
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod LibraryDataRequirements {
    use super::*;
    pub struct Input {
        pub target: Option<FHIRString>,
    }
    pub struct Output {
        pub return_: Library,
    }
}
pub mod ListFind {
    use super::*;
    pub struct Input {
        pub patient: FHIRId,
        pub name: FHIRCode,
    }
    pub struct Output {}
}
pub mod MeasureCareGaps {
    use super::*;
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub topic: FHIRString,
        pub subject: FHIRString,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod MeasureCollectData {
    use super::*;
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub measure: Option<FHIRString>,
        pub subject: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    pub struct Output {
        pub measureReport: MeasureReport,
        pub resource: Option<Vec<Resource>>,
    }
}
pub mod MeasureDataRequirements {
    use super::*;
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
    }
    pub struct Output {
        pub return_: Library,
    }
}
pub mod MeasureEvaluateMeasure {
    use super::*;
    pub struct Input {
        pub periodStart: FHIRDate,
        pub periodEnd: FHIRDate,
        pub measure: Option<FHIRString>,
        pub reportType: Option<FHIRCode>,
        pub subject: Option<FHIRString>,
        pub practitioner: Option<FHIRString>,
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    pub struct Output {
        pub return_: MeasureReport,
    }
}
pub mod MeasureSubmitData {
    use super::*;
    pub struct Input {
        pub measureReport: MeasureReport,
        pub resource: Option<Vec<Resource>>,
    }
    pub struct Output {}
}
pub mod MedicinalProductEverything {
    use super::*;
    pub struct Input {
        pub _since: Option<FHIRInstant>,
        pub _count: Option<FHIRInteger>,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod MessageHeaderProcessMessage {
    use super::*;
    pub struct Input {
        pub content: Bundle,
        pub async_: Option<FHIRBoolean>,
        pub response_url: Option<FHIRUrl>,
    }
    pub struct Output {
        pub return_: Option<Bundle>,
    }
}
pub mod NamingSystemPreferredId {
    use super::*;
    pub struct Input {
        pub id: FHIRString,
        pub type_: FHIRCode,
    }
    pub struct Output {
        pub result: FHIRString,
    }
}
pub mod ObservationLastn {
    use super::*;
    pub struct Input {
        pub max: Option<FHIRPositiveInt>,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod ObservationStats {
    use super::*;
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
    pub struct Output {
        pub statistics: Vec<Observation>,
        pub source: Option<Vec<Observation>>,
    }
}
pub mod PatientEverything {
    use super::*;
    pub struct Input {
        pub start: Option<FHIRDate>,
        pub end: Option<FHIRDate>,
        pub _since: Option<FHIRInstant>,
        pub _type: Option<Vec<FHIRCode>>,
        pub _count: Option<FHIRInteger>,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod PatientMatch {
    use super::*;
    pub struct Input {
        pub resource: Resource,
        pub onlyCertainMatches: Option<FHIRBoolean>,
        pub count: Option<FHIRInteger>,
    }
    pub struct Output {
        pub return_: Bundle,
    }
}
pub mod PlanDefinitionApply {
    use super::*;
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
    pub struct Output {
        pub return_: CarePlan,
    }
}
pub mod PlanDefinitionDataRequirements {
    use super::*;
    pub struct Input {}
    pub struct Output {
        pub return_: Library,
    }
}
pub mod ResourceConvert {
    use super::*;
    pub struct Input {
        pub input: Resource,
    }
    pub struct Output {
        pub output: Resource,
    }
}
pub mod ResourceGraph {
    use super::*;
    pub struct Input {
        pub graph: FHIRUri,
    }
    pub struct Output {
        pub result: Bundle,
    }
}
pub mod ResourceGraphql {
    use super::*;
    pub struct Input {
        pub query: FHIRString,
    }
    pub struct Output {
        pub result: Binary,
    }
}
pub mod ResourceMeta {
    use super::*;
    pub struct Input {}
    pub struct Output {
        pub return_: Meta,
    }
}
pub mod ResourceMetaAdd {
    use super::*;
    pub struct Input {
        pub meta: Meta,
    }
    pub struct Output {
        pub return_: Meta,
    }
}
pub mod ResourceMetaDelete {
    use super::*;
    pub struct Input {
        pub meta: Meta,
    }
    pub struct Output {
        pub return_: Meta,
    }
}
pub mod ResourceValidate {
    use super::*;
    pub struct Input {
        pub resource: Option<Resource>,
        pub mode: Option<FHIRCode>,
        pub profile: Option<FHIRUri>,
    }
    pub struct Output {
        pub return_: OperationOutcome,
    }
}
pub mod StructureDefinitionQuestionnaire {
    use super::*;
    pub struct Input {
        pub identifier_: Option<FHIRString>,
        pub profile: Option<FHIRString>,
        pub url: Option<FHIRString>,
        pub supportedOnly: Option<FHIRBoolean>,
    }
    pub struct Output {
        pub return_: Questionnaire,
    }
}
pub mod StructureDefinitionSnapshot {
    use super::*;
    pub struct Input {
        pub definition: Option<StructureDefinition>,
        pub url: Option<FHIRString>,
    }
    pub struct Output {
        pub return_: StructureDefinition,
    }
}
pub mod StructureMapTransform {
    use super::*;
    pub struct Input {
        pub source: Option<FHIRUri>,
        pub content: Resource,
    }
    pub struct Output {
        pub return_: Resource,
    }
}
pub mod ValueSetExpand {
    use super::*;
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
    pub struct Output {
        pub return_: ValueSet,
    }
}
pub mod ValueSetValidateCode {
    use super::*;
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
    pub struct Output {
        pub result: FHIRBoolean,
        pub message: Option<FHIRString>,
        pub display: Option<FHIRString>,
    }
}
