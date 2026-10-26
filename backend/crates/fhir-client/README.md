# haste-fhir-client

A typed FHIR R4 client interface and an HTTP implementation of it. The `FHIRClient` trait covers every FHIR interaction (read, vread, create, update, conditional update, patch, delete, search, history, operations, batch and transaction) with the resource types from [`haste-fhir-model`](https://crates.io/crates/haste-fhir-model), and the `request` module models each interaction as a `FHIRRequest` and `FHIRResponse` value.

Inside Haste Health the same trait is implemented over HTTP (this crate), over the database (the server's internal client) and by the middleware that wraps either, so `haste-fhir-ops`, `haste-fhir-terminology`, `haste-fhir-profiling`, `haste-testscript-runner`, `haste-access-control` and the CLI all talk FHIR through it.

## Install

```bash
cargo add haste-fhir-client --features http
```

## Usage

```rust
use std::sync::Arc;

use haste_fhir_client::FHIRClient;
use haste_fhir_client::http::{FHIRHttpAuthenticationMethod, FHIRHttpClient, FHIRHttpState};
use haste_fhir_client::url::ParsedParameters;
use haste_fhir_model::r4::generated::resources::ResourceType;

let token = std::env::var("ACCESS_TOKEN")?;
let state = FHIRHttpState::new(
    "https://api.haste.health/w/[tenant]/[project]/api/v1/fhir",
    Some(FHIRHttpAuthenticationMethod::BearerToken(Arc::new(move || {
        let token = token.clone();
        Box::pin(async move { Ok(token) })
    }))),
)?;
let client: FHIRHttpClient<()> = FHIRHttpClient::new(state);

let patient = client.read((), ResourceType::Patient, "123".to_string()).await?;
let bundle = client
    .search_type((), ResourceType::Patient, ParsedParameters::try_from("name=Smith&_count=10")?)
    .await?;
```

The token callback runs on every request, so it can refresh or cache as it likes. `FHIRHttpAuthenticationMethod::Basic` does the same for a username and password. The `CTX` type parameter is passed through to each call; implement `HttpRequestHeaders` on your own context type to add per-request headers (the unit type sends none).

A non-2xx response whose body is an `OperationOutcome` comes back as that `OperationOutcomeError`, and a `202 Accepted` with `Content-Location` comes back as `FHIRResponse::Accepted` for the FHIR asynchronous request pattern. `FHIRHttpExchange` adds `request_with_metadata`, which also returns the status and headers, and `send` for plain requests such as polling that status URL (credentials are only sent to the FHIR base URL's origin).

## Features

| Feature | What it enables                                                                                                                                                                            |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `http`  | `http::FHIRHttpClient`, the reqwest implementation of `FHIRClient`                                                                                                                         |
| `axum`  | `impl IntoResponse for FHIRResponse`: the right status code, `application/fhir+json`, `ETag` and `Last-Modified` from `meta`, `Cache-Control: no-store`, and `202` with `Content-Location` |

## API

- `FHIRClient<CTX, Error>`: one method per FHIR interaction, plus `request` for a `FHIRRequest` value.
- `request`: `FHIRRequest` and `FHIRResponse` enums with a struct per interaction. Their `Debug` output redacts ids and resource bodies.
- `url`: `ParsedParameters` parses a query string into search parameters, with modifiers, chains (`subject:Patient.name:exact`), FHIR escaping, `Segments` for `system|code` values and `Prefix` for `ge2024-01-01` style comparisons.
- `canonical_resolver::CanonicalResolver`: resolve a canonical URL to a conformance resource. Implemented by the server and consumed by the profiling and terminology crates.

## Links

- [API reference](https://rust.haste.health/haste_fhir_client/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/fhir-client)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust) and the [FHIR REST API](https://haste.health/docs/api/rest_api/fhir/intro) it speaks
- Related: [haste-fhir-model](https://crates.io/crates/haste-fhir-model), [haste-fhir-operation-error](https://crates.io/crates/haste-fhir-operation-error)
- TypeScript equivalent: [@haste-health/client](https://www.npmjs.com/package/@haste-health/client)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
