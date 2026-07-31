# Running the Haste Health Binary

The same `haste-health` binary is used whether you built it from source (`cargo build --locked --release`), downloaded a [release binary](https://github.com/HasteHealth/HasteHealth/releases/latest), or are running the [Docker image](https://github.com/HasteHealth/HasteHealth/pkgs/container/hastehealth%2Fhastehealth). Only the invocation changes:

| Context | Invocation                     |
| ------- | ------------------------------ |
| Source  | `cargo run <command>`          |
| Binary  | `./haste-health <command>`     |
| Docker  | `docker run <image> <command>` |

Configuration (`haste.toml` / `HASTE_`-prefixed env vars) is documented separately for the [server](./server_configuration.md) and [worker](./worker_configuration.md); every command below reads the same config.

## `server start`

Starts the FHIR API server.

```bash
haste-health server start [--port <PORT>]
```

- `--port` (`-p`) — defaults to `3000`.

Docker example (from [docker-compose.yml](../../docker-compose.yml)):

```bash
docker run --rm -p 3000:3000 --env-file .env ghcr.io/hastehealth/hastehealth/hastehealth:latest server start
```

## `worker`

Runs the background worker (search indexing and FHIR subscription processing).

```bash
haste-health worker
```

Docker example:

```bash
docker run --rm --env-file .env ghcr.io/hastehealth/hastehealth/hastehealth:latest worker
```

## `admin migrate`

Runs schema/data migrations. Must be run before the first `server start`/`worker`.

```bash
haste-health admin migrate all          # repo + search schema, plus built-in artifacts
haste-health admin migrate repo         # PostgreSQL schema only
haste-health admin migrate search       # Elasticsearch schema only
haste-health admin migrate artifacts    # load built-in/embedded FHIR artifacts
haste-health admin migrate reset-artifacts
```

## `admin tenant create`

Creates a tenant along with its owner user.

```bash
haste-health admin tenant create \
  --id my-health \
  --owner-email myuser@health.org \
  --owner-password testing_password \
  --subscription-tier unlimited
```

## `admin user create`

Creates an additional user on an existing tenant.

```bash
haste-health admin user create --email user@health.org --password testing_password --tenant my-health
```

## `admin client create`

Creates a `ClientApplication` for OAuth client-credentials access.

```bash
haste-health admin client create --id my-client --secret my-secret --tenant my-health --project default
```

## Other commands

`api`, `config`, `testscript`, `hl7v2`, `generate`, and `fhirpath` are development/operator CLI commands (querying FHIR resources, running TestScripts, HL7v2 send/receive, code generation). See the [CLI tutorial](https://haste.health/docs/tutorials/cli) or run `haste-health --help` / `haste-health <command> --help` for full usage.
