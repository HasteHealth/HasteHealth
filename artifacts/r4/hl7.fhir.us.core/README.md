# @haste-health/hl7.fhir.us.core

The HL7 US Core Implementation Guide, STU9 (version 9.0.0, FHIR R4 4.0.1), redistributed unchanged as an npm package: 55 profiles and 15 extensions, 110 search parameters, the US Core value sets and code systems, the `$docref` operation and the server and client CapabilityStatements. US Core defines the minimum a conformant US server must support for patient data and underlies the ONC (g)(10) certification criteria.

Haste Health validates resources against these profiles at write time. The TypeScript validation tests in `@haste-health/fhir-validation` and the Rust `haste-fhir-profiling` crate read this package as their US Core fixture set, and the repository's `backend/scripts/load_us_core.mjs` pushes the whole package into a project as one transaction before the US Core TestScripts run.

## Install

```bash
npm install @haste-health/hl7.fhir.us.core
```

## Usage

To load the profiles in code:

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const profiles = loadArtifacts({
  fhirVersion: R4,
  resourceType: "StructureDefinition",
  currentDirectory: import.meta.dirname,
  onlyPackages: ["@haste-health/hl7.fhir.us.core"],
});
```

To make a Haste Health project enforce US Core, load the StructureDefinitions into the project and reference them from `meta.profile`; the server resolves each canonical URL against the project's StructureDefinitions whenever a resource is created or updated. The [profiling guide](https://haste.health/docs/guides/profiling) covers what is checked.

## Contents

| Resource type       | Count | Notes                                                                                                                          |
| ------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------ |
| StructureDefinition | 70    | 55 profiles (`us-core-patient`, `us-core-blood-pressure`, `us-core-condition-problems-health-concerns`, ...) and 15 extensions |
| SearchParameter     | 110   | US Core's constrained search parameters, for example `us-core-condition-code`                                                  |
| ValueSet            | 21    |                                                                                                                                |
| CodeSystem          | 4     | `us-core-category`, `condition-category`, `careplan-category`, `us-core-documentreference-category`                            |
| CapabilityStatement | 2     | `us-core-server`, `us-core-client`                                                                                             |
| OperationDefinition | 1     | `$docref`                                                                                                                      |
| Basic               | 3     | Certification metadata                                                                                                         |

Files sit flat in the package root, one resource per file, named `<ResourceType>-<id>.json`, and `.index.json` lists them all.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/hl7.fhir.us.core)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4/hl7.fhir.us.core)
- Upstream: [US Core STU9](https://hl7.org/fhir/us/core/STU9/)
- Haste Health: [US Core conformance report](https://haste.health/docs/reference/conformance/us-core), [ONC (g)(10)](https://haste.health/docs/reference/conformance/onc-g10), [Profiling and validation](https://haste.health/docs/guides/profiling)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/hl7.fhir.r4.core](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core)

The FHIR content is HL7 International's, published under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Only the npm packaging is [Haste Health](https://haste.health)'s.
