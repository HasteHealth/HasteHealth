# @haste-health/hl7.fhir.r4.core

The HL7 FHIR R4 (4.0.1) core definitions, repackaged for Haste Health: every StructureDefinition, SearchParameter, OperationDefinition, ValueSet, CodeSystem, ConceptMap, CompartmentDefinition and CapabilityStatement from the specification's definitions download, plus the `fhir.schema.json` JSON Schema. The upstream files are exactly as HL7 published them; next to each sits a `.min.json` build, and that is what consumers load.

This is the base layer under everything FHIR in Haste Health. The server embeds the `.min.json` bundles at build time (the `haste-artifacts` crate) and takes its system-wide search parameters from `search-parameters.min.json`. Rust code generation reads the type, resource and terminology bundles to produce `haste-fhir-model`, and the TypeScript CLI reads the package through `@haste-health/artifacts` to generate `@haste-health/fhir-types`, `@haste-health/meta-value` and `@haste-health/generated-ops`.

## Install

```bash
npm install @haste-health/hl7.fhir.r4.core
```

The package is about 140 MB unpacked because it ships the upstream bundles alongside the minimized ones.

## Usage

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const searchParameters = loadArtifacts({
  fhirVersion: R4,
  resourceType: "SearchParameter",
  currentDirectory: import.meta.dirname,
  onlyPackages: ["@haste-health/hl7.fhir.r4.core"],
});
// 1375 SearchParameters
```

`.index.json` points at the `.min.json` files only, so loaders never see the upstream copies.

## Contents

All files are in `definitions/hl7/`. Counts are for the `.min.json` builds:

| File                             | Contents                                                                                                     |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `profiles-resources.min.json`    | 149 resource StructureDefinitions, 46 OperationDefinitions, 5 CompartmentDefinitions, 2 CapabilityStatements |
| `profiles-types.min.json`        | 63 data type StructureDefinitions                                                                            |
| `profiles-others.min.json`       | 44 profiles                                                                                                  |
| `extension-definitions.min.json` | 393 extension StructureDefinitions                                                                           |
| `dataelements.min.json`          | 6781 data element StructureDefinitions                                                                       |
| `search-parameters.min.json`     | 1375 SearchParameters                                                                                        |
| `valuesets.min.json`             | 672 ValueSets, 495 CodeSystems                                                                               |
| `v2-tables.min.json`             | 428 ValueSets, 424 CodeSystems (HL7 v2 tables)                                                               |
| `v3-codesystems.min.json`        | 216 ValueSets, 143 CodeSystems (HL7 v3)                                                                      |
| `conceptmaps.min.json`           | 19 ConceptMaps                                                                                               |
| `fhir.schema.json`               | The R4 JSON Schema, not indexed                                                                              |

### How the `.min.json` files are built

The upstream `.json` files are never edited, so an HL7 update can be dropped in and every local difference stays recorded in one place: `patches/`. `patches/manifest.toml` names the sources and the Rust rules to run, and every other JSON file in `patches/` is a set of RFC 6902 JSON Patch operations against one targeted resource. The `haste-artifact-patcher` crate applies the patches, runs the rules and minimizes each resource (StructureDefinitions keep only their identifying fields and snapshot). From `backend/`:

```bash
cargo run artifacts build ../artifacts/r4/hl7-core/patches/manifest.toml   # rebuild the .min.json files
cargo run artifacts diff ../artifacts/r4/hl7-core/patches/manifest.toml    # list every change to HL7's resources
```

Today one patch adds Haste Health's own resource types to the `resource-types`, `defined-types` and `all-types` value sets, and the `clippy-doc-markdown` rule rewrites descriptions so the Rust documentation generated from them passes lints. The [artifacts README](https://github.com/HasteHealth/HasteHealth/blob/main/artifacts/Readme.md) describes the pipeline in full.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4/hl7-core)
- Upstream: [HL7 FHIR R4](https://hl7.org/fhir/R4/), [downloads](https://hl7.org/fhir/R4/downloads.html)
- [FHIR resource reference on haste.health](https://haste.health/docs/reference/fhir/model/resources/Patient) (one page per resource type)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/fhir-types](https://www.npmjs.com/package/@haste-health/fhir-types), [@haste-health/haste-health.fhir.r4.core](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4.core)

The FHIR definitions are HL7 International's, published under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). The packaging and patches are part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
