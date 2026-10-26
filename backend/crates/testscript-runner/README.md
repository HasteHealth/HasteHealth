# haste-testscript-runner

Runs FHIR `TestScript` resources against a FHIR server and produces a `TestReport`. It is the engine behind `haste-health testscript run`, which executes Haste Health's conformance suites (core resources, US Core, ONC (g)(10), access control, SQL on FHIR) in CI and feeds the conformance pages on the website. It is server-agnostic: anything reachable through a [`haste-fhir-client`](https://crates.io/crates/haste-fhir-client) HTTP client can be tested.

What a run covers:

- **Fixtures** from `contained` resources, plus the request and response fixtures an operation captures with `requestId` and `responseId`.
- **Variables** (`${name}`) resolved with a FHIRPath `expression` against the fixture `sourceId` names, or from a response's `headerField`. A variable must resolve to a single value.
- **Operations**: `read`, `vread`, `create`, `update`, `updateCreate`, `patch`, `delete`, `deleteCondSingle`, `deleteCondMultiple`, `search`, `history`, `batch`, `transaction`, `capabilities` and `conforms`; `$operation` invocations; and plain HTTP requests (an operation with a `url` and no `type`), which is how a script follows an asynchronous request's status URL.
- **Assertions** on `responseCode` or `response`, `headerField`, `contentType`, `resource` type, and FHIRPath `expression`s compared to a `value` or to `compareToSourceId` with `compareToSourceExpression`, using the operators `equals`, `notEquals`, `in`, `notIn`, `greaterThan`, `lessThan`, `empty`, `notEmpty`, `contains`, `notContains` and `eval`.
- **Index waiting**: search indexing is asynchronous, so before a search, an operation or a conditional delete or update the runner can wait, up to `index_wait_timeout`, for the script's latest writes to become visible.

The `TestReport` mirrors the script's setup, test and teardown actions with a pass or fail result and a message for each failed assertion.

## Install

```bash
cargo add haste-testscript-runner
```

## Usage

```rust
use std::{sync::Arc, time::Duration};

use haste_fhir_client::http::{
    FHIRHttpAuthenticationMethod, FHIRHttpClient, FHIRHttpState, HeaderMap,
    HttpRequestHeaders, WithRequestHeaders,
};
use haste_fhir_model::r4::generated::{resources::TestScript, terminology::ReportResultCodes};
use haste_testscript_runner::{TestRunnerOptions, run};

// The per-request context carries the headers an operation's `requestHeader` asks for.
#[derive(Clone, Debug, Default)]
struct Ctx(Option<HeaderMap>);

impl HttpRequestHeaders for Ctx {
    fn request_headers(&self) -> Option<&HeaderMap> {
        self.0.as_ref()
    }
}

impl WithRequestHeaders for Ctx {
    fn with_request_headers(mut self, headers: HeaderMap) -> Self {
        self.0 = Some(headers);
        self
    }
}

let token = FHIRHttpAuthenticationMethod::BearerToken(Arc::new(|| {
    Box::pin(async { Ok(std::env::var("TOKEN").unwrap_or_default()) })
}));
let client = FHIRHttpClient::<Ctx>::new(FHIRHttpState::new(
    "http://localhost:3000/w/my-tenant/my-project/api/v1/fhir",
    Some(token),
)?);

let script: TestScript = serde_json::from_slice(&std::fs::read("Patient.testscript.json")?)?;

let report = run(
    &client,
    Ctx::default(),
    Arc::new(script),
    Arc::new(TestRunnerOptions {
        wait_between_operations: None,
        index_wait_timeout: Some(Duration::from_secs(10)),
    }),
)
.await?;

assert_eq!(report.result, ReportResultCodes::pass());
```

`run` returns `TestScriptError` when the script itself cannot be executed (a missing fixture, a malformed operation, a failed request). A failing assertion is not an error: the report comes back with `result` set to `fail`.

## Command line

```bash
haste-health testscript run -i backend/testscripts/core -o reports.json --index-wait-ms 10000
```

Runs every TestScript under the input paths in parallel against the active profile's server and writes a transaction `Bundle` of `TestReport`s. `--wait-between-operations-ms` adds a fixed delay after each operation and `--allow-failure <id>` records a known failure without failing the run.

## Links

- [API reference](https://rust.haste.health/haste_testscript_runner/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/testscript-runner)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [Conformance overview](https://haste.health/docs/reference/conformance/overview) and [test coverage](https://haste.health/docs/reference/conformance/test-coverage), the pages built from these reports
- [CLI reference](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/cli_commands.md#haste-health-testscript) and the [FHIR TestScript specification](https://hl7.org/fhir/R4/testscript.html)
- TypeScript equivalent: [@haste-health/testscript-runner](https://www.npmjs.com/package/@haste-health/testscript-runner)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
