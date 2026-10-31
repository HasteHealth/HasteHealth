# @haste-health/testscript-runner

Runs FHIR [TestScript](https://hl7.org/fhir/R4/testscript.html) resources against a FHIR server and returns a [TestReport](https://hl7.org/fhir/R4/testreport.html). It drives the server through [`@haste-health/client`](https://www.npmjs.com/package/@haste-health/client), so it works against any FHIR R4 or R4B endpoint, not only Haste Health.

The `testscript run` command of the Haste Health TypeScript CLI is built on it. Haste Health's own conformance suites run on the Rust equivalent, [haste-testscript-runner](https://crates.io/crates/haste-testscript-runner), and publish their reports on the conformance pages linked below.

## Install

```bash
npm install @haste-health/testscript-runner @haste-health/client pino
```

## Usage

```ts
import pino from "pino";
import createHTTPClient from "@haste-health/client/http";
import { R4 } from "@haste-health/fhir-types/versions";
import { run } from "@haste-health/testscript-runner";

const client = createHTTPClient({
  url: "http://localhost:3000/w/my-tenant/my-project/api/v1/fhir",
  getAccessToken: async () => token,
});

const report = await run(pino(), client, R4, testScript);

report.result; // "pass" or "fail"
report.test?.[0].action; // one TestReport action per TestScript action, with a message per assert
```

`run(logger, client, fhirVersion, testScript, timeout?)` resolves the fixtures, runs `setup`, then every `test`, and always runs `teardown`, even when a step throws. `timeout` is a pause in milliseconds inserted after each operation, useful against servers that index search results asynchronously.

## Supported TestScript features

| Area       | Support                                                                                                                                                                                                                                                                            |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fixtures   | `fixture.resource` pointing at a contained resource (`#id`) or one read from the server (`Type/id`); a `Binary` fixture may carry FHIR JSON or a JSON Patch                                                                                                                        |
| Operations | `read`, `vread`, `create`, `update`, `patch`, `delete`, `search`, `history`, `batch`, `transaction` and `invoke` (operation code in `url`, `Parameters` in `sourceId`), using `resource`, `params`, `sourceId` and `targetId`                                                      |
| Results    | `responseId` and `requestId` store the exchange as fixtures; `sourceId`, `direction` or the latest response select what an assert looks at                                                                                                                                         |
| Variables  | `${name}` in operation values, resolved from `TestScript.variable` entries with `expression` (FHIRPath) and an optional `sourceId`                                                                                                                                                 |
| Asserts    | `resource` (resource type) and `expression` (FHIRPath) compared with `value` or `compareToSourceId` + `compareToSourceExpression` using the FHIR operators `equals`, `notEquals`, `in`, `notIn`, `greaterThan`, `lessThan`, `empty`, `notEmpty`, `contains`, `notContains`, `eval` |

Variables that use `defaultValue`, `headerField` or `path` instead of `expression` are not supported and make the run throw an `OperationError` from [`@haste-health/operation-outcomes`](https://www.npmjs.com/package/@haste-health/operation-outcomes). Assert fields other than `resource` and `expression` (`responseCode`, `headerField`, `path`, `minimumId`, ...) are not evaluated, so an assert that relies only on them passes.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_testscript-runner.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/testscript-runner)
- [Conformance overview](https://haste.health/docs/reference/conformance/overview) and [test coverage](https://haste.health/docs/reference/conformance/test-coverage), the reports produced from Haste Health's TestScripts
- Rust equivalent: [haste-testscript-runner](https://crates.io/crates/haste-testscript-runner)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
