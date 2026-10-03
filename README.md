<div align="center">
   <img src="https://raw.githubusercontent.com/HasteHealth/HasteHealth/refs/heads/main/markdown_assets/banner.svg" style="height: 350px; width: 500px;" />
</div>

## Overview

Haste Health is the open-source FHIR server for AI-native health apps. It stores clinical data as FHIR R4 and serves it to your applications, analytics and AI agents through one API, with OAuth2, SMART on FHIR and MCP built in.

Epic, Oracle Health and HL7v2 in. FHIR, SQL-on-FHIR and MCP out. Written in Rust, licensed Apache-2.0, and yours to self-host, or [hosted with a signed BAA](https://haste.health/pricing).

## Running Locally

The quickest way to get everything running is with the top-level [docker-compose.yml](./docker-compose.yml), which pulls the published images and starts PostgreSQL, the server, the worker, and the admin app.

```bash
curl -O https://raw.githubusercontent.com/HasteHealth/HasteHealth/main/docker-compose.yml
docker compose up
```

Once the containers are healthy, open `http://my-health--system.localhost:3001` and log in with:

- username: `myuser@health.org`
- password: `testing_password`

This tenant and user are created automatically by the migration step in the compose file.

If you're actively developing on the backend or frontend instead of just running published images, see [Running for Development](#running-for-development) below.

## Running for Development

### 1. Services

```bash
docker-compose -f docker-services-compose.yml up
```

This starts a PostgreSQL database and a migration job for the repository and
search index schema migrations.

### 2. Server

Once the services have finished starting, open a **separate terminal** and go to
the `backend` directory:

```bash
cd ./backend
```

then run the server with the following command:

```bash
cargo run server start
```

Configuration for the server can be found [here](./backend/documentation/server_configuration.md).

### 3. Worker

Finally, the worker is the last service to be launched. It handles search
indexing and FHIR subscription processing in the background.

Open a **separate terminal** and go to the `backend` directory:

```bash
cd ./backend
```

then run:

```bash
cargo run worker
```

Configuration for the worker can be found [here](./backend/documentation/worker_configuration.md).

### 4. Frontend

In a **separate terminal**, run the frontend admin application:

```bash
cd <repo-root>/frontend/packages/admin-app
pnpm dev
```

Then go to `http://my-health--system.localhost:3001` and fill in the following credentials:

- username: `myuser@health.org`
- password: `testing_password`

This tenant and user are created automatically when you run the migration in the earlier docker-compose step.

## Binaries

One `haste-health` binary runs the server, the worker and the CLI. Install it with npm:

```bash
npm install -g haste-health
haste-health --help
```

Or try it without installing:

```bash
npx haste-health --help
```

Builds exist for Linux (x64 and arm64) and macOS on Apple Silicon. The Linux builds are static and run on any distribution. On Windows, use it inside WSL. Each [GitHub release](https://github.com/HasteHealth/HasteHealth/releases/latest) also has the binaries attached, for machines without Node.js.

Configuration (`haste.toml` or environment variables) is documented [here for the server](./backend/documentation/server_configuration.md) and [here for the worker](./backend/documentation/worker_configuration.md).

Available commands (`server start`, `worker`, `admin migrate`, `admin tenant create`, etc.) are documented [here](./backend/documentation/cli_commands.md).

## Docker Images

- [Server](https://github.com/HasteHealth/HasteHealth/pkgs/container/hastehealth%2Fhastehealth)
- [Admin App](https://github.com/HasteHealth/HasteHealth/pkgs/container/hastehealth%2Fadmin-app)

Configuration (`haste.toml` or environment variables) is documented [here for the server](./backend/documentation/server_configuration.md) and [here for the worker](./backend/documentation/worker_configuration.md). See [docker-compose.yml](./docker-compose.yml) for a full working setup.

Available commands (`server start`, `worker`, `admin migrate`, `admin tenant create`, etc.) are documented [here](./backend/documentation/cli_commands.md).

## RFCs (Request for Comments)

For large feature requests submit RFCS the following is a guide for viewing/submitting RFCs:

RFCs can be written [here](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/website/docs/rfc/proposals).

They should follow the format specified [here](https://github.com/HasteHealth/HasteHealth/blob/main/frontend/packages/website/docs/rfc/format.mdx).

RFCs can be read [here](https://haste.health/docs/category/rfcs)

## Performance

Using `wrk` for performance testing.

### Example

```bash
wrk --latency -s crates/server/benchmarks/transaction.lua -t10 -c10 -d10s http://localhost:3000/w/ohio-health/zb154qm9/api/v1/fhir
```

#### M3 Macbook Air Local 10 threads Postgres 18

| Latency (percentile:time)                           | Requests per Second                                      | Concurrent connections | Benchmark                                                   |
| --------------------------------------------------- | -------------------------------------------------------- | ---------------------- | ----------------------------------------------------------- |
| 50%:1.2ms, 90%:1.8ms, 99%:3.38                      | 10344                                                    | 10                     | backend/crates/server/benchmarks/observation.lua            |
| 50%:60ms, 90%:73ms, 99%:288.6ms                     | 251 (100 resources per transaction) (25100 total writes) | 10                     | backend/crates/server/benchmarks/transaction.lua            |
| 50%:116.73ms 75%:118.39ms 90%:121.45ms 99%:246.90ms | 325 (100 reads per batch) (32500 total reads)            | 10                     | backend/crates/server/benchmarks/observation_batch_read.lua |
