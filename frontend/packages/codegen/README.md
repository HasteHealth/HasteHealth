# @haste-health/codegen

Code generators that turn FHIR StructureDefinitions and OperationDefinitions into the TypeScript the rest of the Haste Health SDK is built from: the resource types and type sets in [`@haste-health/fhir-types`](https://www.npmjs.com/package/@haste-health/fhir-types), the typed operations in [`@haste-health/generated-ops`](https://www.npmjs.com/package/@haste-health/generated-ops), and the element metadata and type-to-canonical maps in [`@haste-health/meta-value`](https://www.npmjs.com/package/@haste-health/meta-value). [`@haste-health/fhir-validation`](https://www.npmjs.com/package/@haste-health/fhir-validation) also uses its StructureDefinition traversal helpers at run time.

The `generate` commands of the Haste Health TypeScript CLI (`types-artifacts`, `operations`, `meta`, `type-to-canonical`) are thin wrappers around these functions. Each takes the StructureDefinitions or OperationDefinitions loaded with [`@haste-health/artifacts`](https://www.npmjs.com/package/@haste-health/artifacts) and returns generated source as a string.

## Install

```bash
npm install @haste-health/codegen @haste-health/artifacts
```

## Usage

```ts
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { loadArtifacts } from "@haste-health/artifacts";
import { generateSets } from "@haste-health/codegen/generate/sets";
import { generateTypes } from "@haste-health/codegen/generate/typescript-types";
import { R4 } from "@haste-health/fhir-types/versions";

// StructureDefinitions from the FHIR packages in this project's dependencies.
const structureDefinitions = loadArtifacts({
  fhirVersion: R4,
  resourceType: "StructureDefinition",
  currentDirectory: fileURLToPath(import.meta.url),
});

writeFileSync("types.ts", generateTypes(R4, structureDefinitions));
writeFileSync("sets.ts", generateSets(R4, structureDefinitions));
```

Generate from a package of your own profiles the same way: add the FHIR package to your dependencies and the loader picks it up.

## Generators

| Import                                                 | Function                                                                       | Produces                                                                                                                    |
| ------------------------------------------------------ | ------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `@haste-health/codegen/generate/typescript-types`      | `generateTypes(version, structureDefinitions)`                                 | Interfaces for every primitive, complex and resource type, plus the `ResourceType`, `AResource` and `DataType` lookup types |
| `@haste-health/codegen/generate/sets`                  | `generateSets(version, structureDefinitions)`                                  | The `resourceTypes`, `complexTypes` and `primitiveTypes` string sets                                                        |
| `@haste-health/codegen/generate/operation-definition`  | default `operationGeneration(version, operationDefinitions)`, `generateOp(op)` | One namespace per OperationDefinition with `Input`, `Output`, `IOp` and `Op` (R4 only)                                      |
| `@haste-health/codegen/generate/meta-data`             | `generateMetaData(structureDefinitions)`                                       | Compiled element metadata (`MetaV2Compiled`) that `@haste-health/meta-value` uses for type-aware FHIRPath and validation    |
| `@haste-health/codegen/generate/type-to-canonical`     | `generateTypeToCanonicalMap(structureDefinitions)`                             | A map from type name to canonical URL                                                                                       |
| `@haste-health/codegen/traversal/structure-definition` | `traversalBottomUp`, `eleIndexToChildIndices`                                  | Walk a StructureDefinition snapshot as an element tree                                                                      |

The root import, `@haste-health/codegen`, exposes the same modules as `generate.typescript`, `generate.searchParameter` (the sets), `generate.operationDefinition`, `generate.metaData` and `sdTraversal`. Generated files start with a `@generated` comment; change the artifacts or the generator and regenerate rather than editing them.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_codegen.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/codegen)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-codegen](https://crates.io/crates/haste-codegen), which generates the Rust model and operations

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
