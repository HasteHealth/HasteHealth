# @haste-health/artifacts

Loads FHIR conformance resources from the FHIR npm packages installed in your project. A FHIR package such as [`@haste-health/hl7.fhir.r4.core`](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core), [`@haste-health/hl7.fhir.us.core`](https://www.npmjs.com/package/@haste-health/hl7.fhir.us.core) or [`@haste-health/haste-health.fhir.r4.core`](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4.core) ships an `.index.json` that lists every resource file it contains. `loadArtifacts` walks your `package.json` dependencies, keeps the packages whose `fhirVersions` match, and returns the resources of the type you ask for, flattening Bundles on the way. `generateIndexFile` builds that `.index.json` for a package of your own.

This is how the Haste Health code generators ([`@haste-health/codegen`](https://www.npmjs.com/package/@haste-health/codegen) and the CLI), [`@haste-health/meta-value`](https://www.npmjs.com/package/@haste-health/meta-value) and the validation and operation test suites obtain StructureDefinitions, OperationDefinitions, SearchParameters and terminology without a server.

## Install

```bash
npm install @haste-health/artifacts @haste-health/hl7.fhir.r4.core
```

## Usage

```ts
import { fileURLToPath } from "node:url";
import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const structureDefinitions = loadArtifacts({
  fhirVersion: R4,
  resourceType: "StructureDefinition",
  // Any file inside your package: the nearest package.json's dependencies are scanned.
  currentDirectory: fileURLToPath(import.meta.url),
});

const patient = structureDefinitions.find(
  (sd) => sd.url === "http://hl7.org/fhir/StructureDefinition/Patient",
);
```

| Option                    | Meaning                                                                                                                 |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `fhirVersion`             | `R4` or `R4B` from `@haste-health/fhir-types/versions`; a package is used when one of its `fhirVersions` starts with it |
| `resourceType`            | The resource type to load; the return type follows it                                                                   |
| `currentDirectory`        | Where to start searching upwards for the `package.json` whose dependencies are scanned                                  |
| `onlyPackages`            | Restrict to these package names                                                                                         |
| `loadDevelopmentPackages` | Also scan `devDependencies`                                                                                             |
| `silence`                 | Suppress the log line printed per package                                                                               |

### Indexing your own package

```ts
import { writeFileSync } from "node:fs";
import { generateIndexFile } from "@haste-health/artifacts";

const index = generateIndexFile({
  root: "./my-implementation-guide",
  artifactLocations: ["definitions"], // directories under root, walked recursively
  extension: ".json", // default; the HL7 core package indexes ".min.json"
  ignore: ["*.schema.json"], // glob-like patterns, * matches anything
});

writeFileSync(
  "./my-implementation-guide/.index.json",
  JSON.stringify(index, null, 2),
);
```

Every JSON file with a `resourceType` gets an entry with its `filename` and `resourceType`; a Bundle gets one entry per resource type it contains. Add `fhirVersions` (for example `["4.0.1"]`) to the package's `package.json` so `loadArtifacts` recognises it.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_artifacts.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/artifacts), and the FHIR packages themselves under [artifacts/](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- FHIR package conventions: [FHIR NPM Package Spec](https://confluence.hl7.org/display/FHIR/NPM+Package+Specification)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
