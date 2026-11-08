# @haste-health/hl7.fhir.r4b.test-data

Example FHIR R4B (4.3.0) resources for tests: one of the examples HL7 publishes with the R4B specification for each of its 139 resource types, 141 resources in all once the notification `Bundle` example's own `SubscriptionStatus` and `Encounter` entries are counted. Files are named `<ResourceType>-<id>.json`, for example `Patient-animal.json` and `Encounter-emerg.json`, and an `.index.json` lets `@haste-health/artifacts` load them by resource type.

It is the R4B counterpart of [`@haste-health/hl7.fhir.r4.test-data`](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.test-data): a fixture set for tests that need a realistic R4B resource of a given type, such as validating generated types or structural validators against every resource in the specification.

## Install

```bash
npm install --save-dev @haste-health/hl7.fhir.r4b.test-data
```

## Usage

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4B } from "@haste-health/fhir-types/versions";

const encounters = loadArtifacts({
  fhirVersion: R4B,
  resourceType: "Encounter",
  currentDirectory: import.meta.dirname,
  loadDevelopmentPackages: true,
  onlyPackages: ["@haste-health/hl7.fhir.r4b.test-data"],
});
```

`loadArtifacts` only reads `dependencies` unless `loadDevelopmentPackages` is set, so pass it when the package is a dev dependency, which it normally should be.

## Contents

- `examples-json/`: 140 JSON files, 141 resources, one per R4B resource type from `Account` to `VisionPrescription`, including the R4B additions such as `SubscriptionTopic`, `SubscriptionStatus`, `Citation`, `EvidenceReport` and the medication definition resources.
- `.index.json`: one entry per JSON file with its `resourceType`.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4b.test-data)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4b/test-data)
- Upstream: [HL7 FHIR R4B downloads](https://hl7.org/fhir/R4B/downloads.html) (the examples archive)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/hl7.fhir.r4b.core](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4b.core), [@haste-health/hl7.fhir.r4.test-data](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.test-data)

The example resources are HL7 International's, published under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). The packaging is part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
