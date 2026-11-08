# @haste-health/hl7.fhir.r4.test-data

Example FHIR R4 (4.0.1) resources for tests: the examples HL7 publishes with the R4 specification, mostly one per resource type, covering 67 resource types from `Account` to `ValueSet`. They are the files from the specification's examples download (`patient-example.json`, `observation-example.json`, `medicationrequest0301.json` and so on), with an `.index.json` so that `@haste-health/artifacts` can load them by resource type.

Haste Health uses them as fixtures. `@haste-health/fhir-validation` validates every example against the HL7 core StructureDefinitions in its structural and profile tests, and they are the sample inputs whenever a test needs a realistic resource of a given type.

## Install

```bash
npm install --save-dev @haste-health/hl7.fhir.r4.test-data
```

## Usage

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const patients = loadArtifacts({
  fhirVersion: R4,
  resourceType: "Patient",
  currentDirectory: import.meta.dirname,
  loadDevelopmentPackages: true,
  onlyPackages: ["@haste-health/hl7.fhir.r4.test-data"],
});
```

`loadArtifacts` only reads `dependencies` unless `loadDevelopmentPackages` is set, so pass it when the package is a dev dependency, which it normally should be.

## Contents

- `examples-json/`: 69 JSON files holding 86 resources (a few files are Bundles whose entries count individually) across 67 resource types. Observation has three examples and Organization two; every other type has one. The folder also holds 10 XML copies and 21 `.json.out` files, which are not indexed and not loaded.
- `.index.json`: one entry per JSON file with its `resourceType`.

Resource types with an example: Account, AllergyIntolerance, Appointment, AppointmentResponse, AuditEvent, Basic, Binary, BodyStructure, CapabilityStatement, CareTeam, ClinicalImpression, CodeSystem, Communication, CompartmentDefinition, Composition, ConceptMap, Condition, Consent, DetectedIssue, Device, DeviceMetric, DeviceUseStatement, DiagnosticReport, DocumentReference, Encounter, Endpoint, EpisodeOfCare, FamilyMemberHistory, Flag, Goal, GraphDefinition, Group, HealthcareService, Immunization, ImplementationGuide, Linkage, List, Location, MedicationAdministration, MedicationDispense, MedicationRequest, MedicationStatement, MessageDefinition, MessageHeader, MolecularSequence, NamingSystem, Observation, OperationDefinition, OperationOutcome, Organization, Parameters, Patient, PaymentNotice, Person, Practitioner, PractitionerRole, Questionnaire, QuestionnaireResponse, RiskAssessment, Schedule, Slot, Specimen, StructureMap, Subscription, Substance, SupplyDelivery, ValueSet.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.test-data)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4/test-data)
- Upstream: [HL7 FHIR R4 downloads](https://hl7.org/fhir/R4/downloads.html) (the examples archive)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/hl7.fhir.r4.core](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core), [@haste-health/hl7.fhir.r4b.test-data](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4b.test-data)

The example resources are HL7 International's, published under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). The packaging is part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
