# @haste-health/generated-ops

Typed operation objects for every FHIR R4 OperationDefinition that Haste Health ships: the HL7 core operations (`$expand`, `$validate-code`, `$lookup`, `$everything`, `$validate`, `$lastn`, `$match`, `$snapshot`, ...) and the Haste Health operations (`$invite-user`, `$evaluate-policy`, `$hl7v2-parse`, `$run` on a ViewDefinition, `$tenant-branding`, ...). Each is an `Operation` from [`@haste-health/operation-execution`](https://www.npmjs.com/package/@haste-health/operation-execution), so its `Input` and `Output` types are known at compile time and its parameters are validated before and after the call.

[`@haste-health/client`](https://www.npmjs.com/package/@haste-health/client) accepts these objects in `invoke_system`, `invoke_type` and `invoke_instance`. The Haste Health admin app, CLI and [`@haste-health/components`](https://www.npmjs.com/package/@haste-health/components) call the server through them.

## Install

```bash
npm install @haste-health/generated-ops @haste-health/client
```

## Usage

```ts
import createHTTPClient from "@haste-health/client/http";
import { R4 } from "@haste-health/fhir-types/versions";
import type { uri } from "@haste-health/fhir-types/r4/types";
import {
  PatientEverything,
  ValueSetExpand,
} from "@haste-health/generated-ops/r4";

const client = createHTTPClient({
  url: "http://localhost:3000/w/my-tenant/my-project/api/v1/fhir",
  getAccessToken: async () => token,
});

// Type level: POST ValueSet/$expand. The input is ValueSetExpand.Input and
// the result ValueSetExpand.Output, a ValueSet.
const expansion = await client.invoke_type(
  ValueSetExpand.Op,
  {},
  R4,
  "ValueSet",
  {
    url: "http://hl7.org/fhir/ValueSet/administrative-gender" as uri,
  },
);

// Instance level: POST Patient/123/$everything.
const bundle = await client.invoke_instance(
  PatientEverything.Op,
  {},
  R4,
  "Patient",
  "123",
  {},
);
```

Every export is a namespace named after the OperationDefinition `id` in PascalCase (`ValueSet-expand` becomes `ValueSetExpand`, `haste-health-invite-user` becomes `HasteHealthInviteUser`) with four members:

| Member   | Meaning                                                                          |
| -------- | -------------------------------------------------------------------------------- |
| `Input`  | Object type of the `in` parameters; arrays where `max` is `*`, nested for `part` |
| `Output` | Object type of the `out` parameters, or the returned resource type               |
| `Op`     | The `Operation<Input, Output>` instance, with the full `operationDefinition`     |
| `IOp`    | `IOperation<Input, Output>`, for functions that take the operation as a value    |

For an operation the server supports but this package does not define, the client's `invoke_*` methods also accept the operation code as a string, without typing or validation.

## Regenerating

`src/r4/ops.ts` is generated. From `frontend/`, `pnpm --filter @haste-health/generated-ops run generate` runs the `generate operations` command of the Haste Health CLI over the OperationDefinitions in the FHIR artifact packages (`@haste-health/hl7.fhir.r4.core` and `@haste-health/haste-health.fhir.r4.core`) and rewrites the file. Add or change an OperationDefinition in the artifacts and regenerate rather than editing it by hand. Only R4 is generated at present.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_generated-ops.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/generated-ops)
- [FHIR REST API: invoking operations](https://haste.health/docs/api/rest_api/fhir/intro) and the [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-fhir-generated-ops](https://crates.io/crates/haste-fhir-generated-ops)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
