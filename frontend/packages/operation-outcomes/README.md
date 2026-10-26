# @haste-health/operation-outcomes

Builds FHIR [OperationOutcome](https://hl7.org/fhir/R4/operationoutcome.html) resources and throws them as a typed JavaScript error. The Haste Health TypeScript packages that can fail with a FHIR-shaped error ([`@haste-health/client`](https://www.npmjs.com/package/@haste-health/client), [`@haste-health/fhir-validation`](https://www.npmjs.com/package/@haste-health/fhir-validation), [`@haste-health/operation-execution`](https://www.npmjs.com/package/@haste-health/operation-execution), [`@haste-health/fhir-pointer`](https://www.npmjs.com/package/@haste-health/fhir-pointer), [`@haste-health/hl7v2-parsing`](https://www.npmjs.com/package/@haste-health/hl7v2-parsing) and [`@haste-health/testscript-runner`](https://www.npmjs.com/package/@haste-health/testscript-runner)) all report problems through it, so one `catch` handles them all.

## Install

```bash
npm install @haste-health/operation-outcomes
```

## Usage

```ts
import {
  OperationError,
  isOperationError,
  issueToStatusCode,
  outcomeError,
} from "@haste-health/operation-outcomes";

function readPatient(id: string) {
  if (!id) {
    throw new OperationError(
      outcomeError("not-found", `Patient '${id}' not found`),
    );
  }
}

try {
  readPatient("");
} catch (e) {
  if (isOperationError(e)) {
    const status = issueToStatusCode(e.outcome.issue[0]); // 404
    respond(status, e.outcome); // the OperationOutcome resource
  }
}
```

## API

| Export                                                          | Purpose                                                                                                                                           |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `OperationError`                                                | `Error` subclass carrying an `OperationOutcome`, read back through `.outcome`                                                                     |
| `isOperationError(e)`                                           | Type guard for `catch` blocks                                                                                                                     |
| `issue(severity, code, diagnostics, expression?)`               | One `OperationOutcome.issue` entry                                                                                                                |
| `issueError`, `issueFatal`, `issueWarning`, `issueInfo`         | `issue` with the severity fixed                                                                                                                   |
| `outcome(issues)`                                               | Wraps issues in an `OperationOutcome` resource                                                                                                    |
| `outcomeError`, `outcomeFatal`, `outcomeWarning`, `outcomeInfo` | A single-issue `OperationOutcome` in one call                                                                                                     |
| `issueToStatusCode(issue)`                                      | HTTP status for an issue: `fatal` 500; `error` 400, or 404 for `not-found`, 401 for `login`, 403 for `forbidden`; `warning` and `information` 200 |

`code` takes the FHIR [IssueType](https://hl7.org/fhir/R4/valueset-issue-type.html) codes (`invalid`, `required`, `not-found`, `forbidden`, `not-supported`, `conflict`, ...), and `expression` is the list of FHIRPath or JSON pointer locations the issue refers to.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_operation-outcomes.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/operation-outcomes)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript)
- Rust equivalent: [haste-fhir-operation-error](https://crates.io/crates/haste-fhir-operation-error)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
