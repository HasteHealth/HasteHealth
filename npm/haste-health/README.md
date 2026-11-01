# haste-health

The [Haste Health](https://haste.health) CLI and FHIR server as one binary. This package installs the prebuilt binary for your platform; the same executable runs the server, the background worker and the command-line client.

## Install

```bash
npm install -g haste-health
haste-health --help
```

Or try it without installing:

```bash
npx haste-health --help
```

Builds exist for Linux (x64 and arm64) and macOS on Apple Silicon, and npm installs only the one for your machine. The Linux builds are static, so they run on any distribution. Each [GitHub release](https://github.com/HasteHealth/HasteHealth/releases/latest) also has the binaries attached, for machines without Node.js.

### Windows

Run haste-health inside [WSL](https://learn.microsoft.com/windows/wsl/install):

1. In PowerShell, install WSL: `wsl --install`
2. Open the WSL terminal, install Node.js there, and run `npm install -g haste-health`.

A server started in WSL is reachable from Windows on `localhost`.

## Commands

| Command                     | What it does                                                                        |
| --------------------------- | ----------------------------------------------------------------------------------- |
| `haste-health server start` | Run the FHIR server (PostgreSQL storage, OAuth2, SMART on FHIR, MCP)                |
| `haste-health worker`       | Run the background workers: search indexing and async jobs                          |
| `haste-health admin`        | Server-side administration: migrations, tenants, users, client applications         |
| `haste-health login`        | Log in as a user through the browser (authorization code + PKCE)                    |
| `haste-health profile`      | Manage named server connection profiles used by the client commands                 |
| `haste-health api`          | Make FHIR REST calls (create, read, search, update, ...) against the active profile |
| `haste-health testscript`   | Run FHIR TestScript resources against the active profile's server                   |
| `haste-health fhirpath`     | Evaluate a FHIRPath expression against a resource read from stdin                   |
| `haste-health hl7v2`        | Bridge HL7v2 messages over MLLP to and from the FHIR server                         |

`haste-health <command> --help` lists the options of each command. Server and worker settings come from `haste.toml` or environment variables.

## Links

- [Setting up the CLI](https://haste.health/docs/tutorials/cli)
- [Command reference](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/cli_commands.md)
- [Server configuration](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/server_configuration.md) and [worker configuration](https://github.com/HasteHealth/HasteHealth/blob/main/backend/documentation/worker_configuration.md)
- [Quick start](https://haste.health/docs/getting_started/quick_start) with Docker Compose
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/npm) for this package; the binary is built from [backend/](https://github.com/HasteHealth/HasteHealth/tree/main/backend)
- The Rust crates behind the binary: [Rust SDK overview](https://haste.health/docs/api/sdks/rust)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
