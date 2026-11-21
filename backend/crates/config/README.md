# haste-config

A small configuration abstraction for the Haste Health binaries: a `Config` trait with `get` and `set`, and one implementation that reads process environment variables after loading `.env` and `.env.development` with [dotenvy](https://crates.io/crates/dotenvy). Failures are `OperationOutcomeError`s from [`haste-fhir-operation-error`](https://crates.io/crates/haste-fhir-operation-error), so a missing variable reports like any other FHIR error.

The `haste-health` binary reads its process-level settings through it: the Sentry DSN, the OpenTelemetry endpoint and headers, and the log format. The FHIR server's own settings (`haste.toml` and `HASTE_`-prefixed variables) are loaded separately with Figment and documented in the links below.

## Install

```bash
cargo add haste-config
```

## Usage

```rust
use haste_config::{Config, ConfigType, get_config};

// Loads `.env` and `.env.development` when present, then reads the process environment.
let config = get_config(ConfigType::Environment);

let log_type: String = config.get("LOG_TYPE")?;
config.set("LOG_TYPE", "JSON".to_string())?;
```

`get_config` returns an `Arc<dyn Config<Key>>`, where `Key` is anything that converts into a `String`: a `&str`, or your own enum of variable names with a `From` impl. `get` fails with an `OperationOutcomeError` (issue type `invalid`) when the variable is not set, and `set` writes the process environment. `ConfigType` has a single variant, `Environment`, and also converts from the string `"environment"`.

## Links

- [API reference](https://rust.haste.health/haste_config/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/config)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [Server configuration reference](https://haste.health/docs/reference/configuration), and the [server](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/server_configuration.md) and [worker](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/worker_configuration.md) configuration docs for the settings the FHIR server itself reads

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
