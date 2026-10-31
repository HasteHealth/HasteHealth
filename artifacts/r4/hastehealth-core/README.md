# @haste-health/haste-health.fhir.r4.core

The FHIR R4 (4.0.1) conformance resources that define what is specific to a Haste Health server: the StructureDefinitions for its own resource types (`User`, `Membership`, `Project`, `ClientApplication`, `IdentityProvider`, the access and de-identification policies, `HL7V2`), the OperationDefinitions for its custom operations, the search parameters on those types, and the code systems and value sets they bind to. It also carries the SQL on FHIR `ViewDefinition` resource and its `$viewdefinition-run` operation.

The package follows the FHIR NPM package layout: `package.json` declares `fhirVersions` and a canonical, and `.index.json` lists every resource file. The Haste Health server embeds these definitions at build time (the `haste-artifacts` crate), Rust and TypeScript code generation read them to produce `haste-fhir-model` and `@haste-health/fhir-types`, and the TypeScript packages load them at runtime through `@haste-health/artifacts`.

## Install

```bash
npm install @haste-health/haste-health.fhir.r4.core
```

## Usage

`loadArtifacts` reads the `.index.json` of every dependency that declares a matching FHIR version, so add this package to your dependencies and ask for a resource type:

```ts
import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const structureDefinitions = loadArtifacts({
  fhirVersion: R4,
  resourceType: "StructureDefinition",
  currentDirectory: import.meta.dirname,
  onlyPackages: ["@haste-health/haste-health.fhir.r4.core"],
});
```

Drop `onlyPackages` to load the HL7 definitions from `@haste-health/hl7.fhir.r4.core` in the same call.

## Contents

| Resource type       | Count | Notes                                                                                                                                                                                                                                                                                                                                       |
| ------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| StructureDefinition | 12    | `User`, `Membership`, `Project`, `ClientApplication`, `IdentityProvider`, `AccessPolicyV2`, `AccessPolicyV2Assignment`, `AccessPolicyV3`, `AccessPolicyV3Assignment`, `DeidentificationPolicy`, `HL7V2`, `ViewDefinition`                                                                                                                   |
| OperationDefinition | 21    | 15 the server implements: `$current-project`, `$current-tenant`, `$endpoints`, `$evaluate-policy`, `$hl7v2-parse`, `$indexing-errors`, `$launch`, `$registration-info`, `$scopes`, `$delete-scope`, `$refresh-tokens`, `$delete-refresh-token`, `$tenant-customization`, `$tenant-branding`, `$viewdefinition-run`; 6 frontend-only (below) |
| SearchParameter     | 27    | For example `User` `email` and `role`, `Membership` `user` and `link`, `ClientApplication` `grantType`, and `_haste-health-author` on every resource                                                                                                                                                                                        |
| CodeSystem          | 38    | Policy, grant type, role, identity provider and SQL on FHIR codes, plus stubs for ISO 3166, ISO 4217 and BCP 13                                                                                                                                                                                                                             |
| ValueSet            | 34    | One per code system, bound from the StructureDefinitions above                                                                                                                                                                                                                                                                              |

Files live under `definitions/haste-health/` (`structure_definition`, `operation`, `search_parameter`, `terminology`) and `definitions/sql-on-fhir/`. The `operation-frontend-only` folder holds OperationDefinitions used only by the TypeScript packages and the admin app (`$deploy`, `$encrypt`, `$invite-user`, `$message-post`, `$password-reset`, `$usage-statistics`); the server neither embeds nor implements them.

## Links

- [npm](https://www.npmjs.com/package/@haste-health/haste-health.fhir.r4.core)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/artifacts/r4/hastehealth-core)
- Resource reference on haste.health: [User](https://haste.health/docs/reference/fhir/model/resources/User), [Membership](https://haste.health/docs/reference/fhir/model/resources/Membership), [Project](https://haste.health/docs/reference/fhir/model/resources/Project), [ClientApplication](https://haste.health/docs/reference/fhir/model/resources/ClientApplication), [IdentityProvider](https://haste.health/docs/reference/fhir/model/resources/IdentityProvider), [AccessPolicyV2](https://haste.health/docs/reference/fhir/model/resources/AccessPolicyV2), [ViewDefinition](https://haste.health/docs/reference/fhir/model/resources/ViewDefinition)
- Guides: [Access control](https://haste.health/docs/auth/authorization/access_control), [SQL on FHIR](https://haste.health/docs/guides/sql_on_fhir)
- Related packages: [@haste-health/artifacts](https://www.npmjs.com/package/@haste-health/artifacts), [@haste-health/fhir-types](https://www.npmjs.com/package/@haste-health/fhir-types), [@haste-health/hl7.fhir.r4.core](https://www.npmjs.com/package/@haste-health/hl7.fhir.r4.core)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
