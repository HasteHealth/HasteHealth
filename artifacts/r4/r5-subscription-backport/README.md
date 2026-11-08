# @haste-health/hl7.fhir.uv.subscriptions-backport.r4

The HL7 Subscriptions R5 Backport Implementation Guide for FHIR R4, redistributed as an npm package. The backport lets an R4 server offer R5-style topic-based subscriptions: a `Subscription` profile with extensions for the topic, filter criteria, heartbeat, timeout and payload content, the R4 `SubscriptionStatus` and notification `Bundle` profiles, the `$status`, `$events`, `$resend` and `$get-ws-binding-token` operations, and extra search parameters on `Subscription`.

This is a snapshot of the IG's continuous build from 17 June 2024 (`notForPublication` in `package.json`), not a balloted release. Haste Health's R4 subscriptions implement it: the server embeds these definitions (the `haste-artifacts` crate) and generates its Rust operation types from the `operation_definition` folder, and the worker's subscription processor derives its triggers from either a plain R4 `Subscription` or a backport one.

## Install

```bash
npm install @haste-health/hl7.fhir.uv.subscriptions-backport.r4
```

The package depends on `@haste-health/hl7.fhir.r4.core`, which the profiles derive from.

## Usage

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const backportProfiles = loadArtifacts({
  fhirVersion: R4,
  resourceType: "StructureDefinition",
  currentDirectory: import.meta.dirname,
  onlyPackages: ["@haste-health/hl7.fhir.uv.subscriptions-backport.r4"],
});
```

## Contents

| Resource type       | Count | Notes                                                                                                                                        |
| ------------------- | ----- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| StructureDefinition | 14    | `backport-subscription`, `backport-subscription-status-r4`, `backport-subscription-notification-r4` and the `backport-*` extensions          |
| OperationDefinition | 4     | `backport-subscription-status`, `backport-subscription-events`, `backport-subscription-resend`, `backport-subscription-get-ws-binding-token` |
| SearchParameter     | 5     | `Subscription` by `topic`, `filter-criteria`, `payload-type`, `custom-channel` and `identifier`                                              |
| CodeSystem          | 1     | `backport-content-code-system`                                                                                                               |
| ValueSet            | 1     | `backport-content-value-set`                                                                                                                 |

Files are grouped as `profiles/`, `operation_definition/` and `parameters/`, with the two terminology files in the package root.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/hl7.fhir.uv.subscriptions-backport.r4)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4/r5-subscription-backport)
- Upstream: [Subscriptions R5 Backport IG](https://hl7.org/fhir/uv/subscriptions-backport/), [continuous build](https://build.fhir.org/ig/HL7/fhir-subscription-backport-ig/)
- Haste Health: [FHIR R4 subscriptions RFC](https://haste.health/docs/rfc/proposals/fhir_r4_subscriptions), [Subscription resource reference](https://haste.health/docs/reference/fhir/model/resources/Subscription)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/hl7.fhir.r4.core](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core)

The FHIR content is HL7 International's, published under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Only the npm packaging is [Haste Health](https://haste.health)'s.
