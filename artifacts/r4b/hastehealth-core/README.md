# @haste-health/haste-health.fhir.r4b.core

Haste Health's own FHIR R4B (4.3.0) conformance resources. Today it holds three stub `CodeSystem` resources for code systems that FHIR binds to but that HL7 publishes no `CodeSystem` resource for: media types (`urn:ietf:bcp:13`, used by `Attachment.contentType`), ISO 3166 countries (`urn:iso:std:iso:3166`) and ISO 4217 currencies (`urn:iso:std:iso:4217`, used by `Money.currency`). Each has `content: not-present`, so it declares that the system exists and resolves bindings without enumerating codes.

It is the R4B counterpart of [`@haste-health/haste-health.fhir.r4.core`](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4.core), which carries the same three stubs under `terminology/external` alongside the Haste Health resource types and operations. The Haste Health server is an R4 server and does not embed this package; it exists for R4B tooling built on `@haste-health/artifacts`.

## Install

```bash
npm install @haste-health/haste-health.fhir.r4b.core
```

## Usage

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4B } from "@haste-health/fhir-types/versions";

const codeSystems = loadArtifacts({
  fhirVersion: R4B,
  resourceType: "CodeSystem",
  currentDirectory: import.meta.dirname,
  onlyPackages: ["@haste-health/haste-health.fhir.r4b.core"],
});
// bcp-13, iso-3166, iso-4217
```

## Contents

| Resource type | Count | Notes                                                                                         |
| ------------- | ----- | --------------------------------------------------------------------------------------------- |
| CodeSystem    | 3     | `bcp-13` Media Types, `iso-3166` Countries, `iso-4217` Currencies, all `content: not-present` |

Files live in `definitions/terminology/external/`, and `.index.json` lists them.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4b.core)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4b/hastehealth-core)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/hl7.fhir.r4b.core](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4b.core), [@haste-health/haste-health.fhir.r4.core](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4.core)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
