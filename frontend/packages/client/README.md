# @haste-health/client

A typed FHIR client for TypeScript and JavaScript. Every FHIR interaction (read, search, create, update, patch, delete, history, operations, batch and transaction) is a method on one `AsynchronousClient`, typed against the R4 and R4B definitions in [`@haste-health/fhir-types`](https://www.npmjs.com/package/@haste-health/fhir-types). The bundled HTTP client talks to any FHIR server over `fetch`; the client itself is transport-agnostic middleware, so the same interface can wrap other transports.

The Haste Health admin app, CLI, React components and TestScript runner all use it.

## Install

```bash
npm install @haste-health/client
```

## Usage

```ts
import createHTTPClient from "@haste-health/client/http";
import type { Patient } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

const client = createHTTPClient({
  url: "https://<host>/w/<tenant>/<project>/api/v1/fhir",
  getAccessToken: async () => accessToken,
});

const created = await client.create({}, R4, {
  resourceType: "Patient",
  name: [{ family: "Smith", given: ["John"] }],
} as Patient);

const { resources, total } = await client.search_type(
  {},
  R4,
  "Patient",
  "name=Smith&_count=10",
);

const patient = await client.read({}, R4, "Patient", created.id!);
```

The first argument of every method is the per-call context. For the HTTP client it is `{ headers?: Record<string, string> }`, so extra headers can be sent on one request.

### HTTP client options

| Option           | Purpose                                                                                                                      |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `url`            | The FHIR base URL, or a function from FHIR version to URL                                                                    |
| `getAccessToken` | Called before each request; the result is sent as a `Bearer` token                                                           |
| `authenticate`   | Called when the server answers 401, for example to start a login flow                                                        |
| `searchMethod`   | `"GET"` (default) puts search parameters in the query string; `"POST"` form-posts them to `_search` to keep them out of URLs |

Responses with status 400 or above throw a `ResponseError` (test with `isResponseError`) carrying the request and the server's `OperationOutcome`.

### Methods

| Method                                               | FHIR interaction                                                                                                |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `capabilities`                                       | `GET metadata`                                                                                                  |
| `read`, `vread`                                      | Read and version read                                                                                           |
| `search_type`, `search_system`                       | Search; parameters as a query string or `ParsedParameter[]`; returns `{ total, resources }`                     |
| `create`, `update`, `conditionalUpdate`, `patch`     | Writes; `patch` takes a JSON Patch body                                                                         |
| `delete_instance`, `delete_type`, `delete_system`    | Deletes, including conditional ones                                                                             |
| `history_instance`, `history_type`, `history_system` | History; returns the Bundle entries                                                                             |
| `invoke_instance`, `invoke_type`, `invoke_system`    | Operations; pass a code such as `"expand"`, or an operation from `@haste-health/generated-ops` for typed in/out |
| `batch`, `transaction`                               | Bundle submission                                                                                               |
| `request`                                            | Send a raw `FHIRRequest` and get the `FHIRResponse`                                                             |

### Middleware

A client is an `AsynchronousClient` built on a middleware chain. Each middleware receives `(state, context, next)`, where `context` holds the `ctx`, the `FHIRRequest` and, after `next`, the `FHIRResponse`, and returns `[state, context]`. This is how the HTTP transport is implemented, and how you add caching, logging or a different transport:

```ts
import { AsynchronousClient } from "@haste-health/client";
import { createMiddlewareAsync } from "@haste-health/client/middleware";

const client = new AsynchronousClient(
  createMiddlewareAsync({}, [
    async (state, context, next) => {
      console.log(context.request.type);
      return next(state, context);
    },
    async (state, context) => [state, { ...context, response: myResponse }],
  ]),
);
```

`@haste-health/client/url` exports `parseUrl` and `parseQuery` for FHIR search strings, plus the escaping helpers (`escapeParameter`, `splitParameter`) that honour FHIR's `\` escapes for `,`, `|` and `$`. `@haste-health/client/types` and `@haste-health/client/interface` hold the request, response and client interface types.

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_client.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/client)
- [TypeScript SDK overview](https://haste.health/docs/api/sdks/typescript) and the [FHIR REST API](https://haste.health/docs/api/rest_api/fhir/intro) on haste.health
- Related: [@haste-health/generated-ops](https://www.npmjs.com/package/@haste-health/generated-ops) for typed operations
- Rust equivalent: [haste-fhir-client](https://crates.io/crates/haste-fhir-client)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
