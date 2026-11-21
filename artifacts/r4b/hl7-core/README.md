# @haste-health/hl7.fhir.r4b.core

The HL7 FHIR R4B (4.3.0) core definitions, repackaged for Haste Health: every StructureDefinition, SearchParameter, OperationDefinition, ValueSet, CodeSystem, ConceptMap, CompartmentDefinition and CapabilityStatement from the specification's definitions download. Each upstream bundle sits next to a `.min.json` build, and the `.min.json` files are what consumers load.

The Haste Health CLI reads this package through `@haste-health/artifacts` to generate the R4B half of `@haste-health/fhir-types` and `@haste-health/meta-value`. The Haste Health server is an R4 server and does not embed it; use [`@haste-health/hl7.fhir.r4.core`](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core) for R4.

## Install

```bash
npm install @haste-health/hl7.fhir.r4b.core
```

The package is about 87 MB unpacked because it ships the upstream bundles alongside the minimized ones.

## Usage

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4B } from "@haste-health/fhir-types/versions";

const structureDefinitions = loadArtifacts({
  fhirVersion: R4B,
  resourceType: "StructureDefinition",
  currentDirectory: import.meta.dirname,
  onlyPackages: ["@haste-health/hl7.fhir.r4b.core"],
});
```

## Contents

All files are in `definitions/`. Counts are for the `.min.json` builds:

| File                             | Contents                                                                                                     |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `profiles-resources.min.json`    | 143 resource StructureDefinitions, 46 OperationDefinitions, 5 CompartmentDefinitions, 2 CapabilityStatements |
| `profiles-types.min.json`        | 64 data type StructureDefinitions                                                                            |
| `profiles-others.min.json`       | 39 profiles                                                                                                  |
| `extension-definitions.min.json` | 398 extension StructureDefinitions                                                                           |
| `dataelements.min.json`          | 274 data element StructureDefinitions                                                                        |
| `search-parameters.min.json`     | 1414 SearchParameters                                                                                        |
| `valuesets.min.json`             | 721 ValueSets, 540 CodeSystems                                                                               |
| `conceptmaps.min.json`           | 17 ConceptMaps                                                                                               |
| `fhir.schema.json.zip`           | The R4B JSON Schema, zipped and not indexed                                                                  |

R4B no longer ships the HL7 v2 tables and v3 code systems; those moved to [terminology.hl7.org](https://terminology.hl7.org), so this package has no `v2-tables` or `v3-codesystems` file.

### How the `.min.json` files are built

This package predates the patch pipeline used by `@haste-health/hl7.fhir.r4.core`. Its upstream bundles were adjusted in place and the edits recorded next to them: `fixes.patch` rewrites search parameter expressions in `search-parameters.json` from `value as CodeableConcept` to `value.ofType(CodeableConcept)`, and `publisher_fix.patch` with `fix.mjs` (`pnpm fix`) fills in a `publisher` and a placeholder `description` on the ValueSets and CodeSystems in `valuesets.json` that lacked one. `pnpm minify` then runs the CLI's `minimize artifacts` over `definitions/` to write the `.min.json` files, and `pnpm generate-index` rebuilds `.index.json` from them.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4b.core)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4b/hl7-core)
- Upstream: [HL7 FHIR R4B](https://hl7.org/fhir/R4B/), [downloads](https://hl7.org/fhir/R4B/downloads.html)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/fhir-types](https://www.npmjs.com/package/@haste-health/fhir-types), [@haste-health/haste-health.fhir.r4b.core](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4b.core)

The FHIR definitions are HL7 International's, published under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). The packaging and patches are part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
