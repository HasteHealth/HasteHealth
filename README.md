<div align="center">
   <img src="https://raw.githubusercontent.com/HasteHealth/HasteHealth/refs/heads/main/markdown_assets/banner.svg" style="height: 350px; width: 500px;" />
</div>

## Overview

Haste Health is a headless EHR — store clinical data as FHIR R4 and serve it to your applications, analytics, and AI agents. MCP-native, multi-tenant, and fast.
Self-host or use a managed cloud.

## Running Locally

The quickest way to get everything running is with the top-level [docker-compose.yml](./docker-compose.yml), which pulls the published images and starts PostgreSQL, Elasticsearch, the server, the worker, and the admin app:

```bash
docker-compose up
```

Once the containers are healthy, open `http://my-health_system.localhost:3001` and log in with:

- username: `myuser@health.org`
- password: `testing_password`

This tenant and user are created automatically by the migration step in the compose file.

If you're actively developing on the backend or frontend instead of just running published images, see [Running for Development](#running-for-development) below.

## Running for Development

### 1. Services

```bash
docker-compose -f docker-services-compose.yml up
```

This starts a PostgreSQL database, Elasticsearch, and a migration job for PostgreSQL and Elasticsearch schema migrations.

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

Then go to `http://my-health_system.localhost:3001` and fill in the following credentials:

- username: `myuser@health.org`
- password: `testing_password`

This tenant and user are created automatically when you run the migration in the earlier docker-compose step.

## Binaries

- [Linux](https://github.com/HasteHealth/HasteHealth/releases/latest/download/haste-health_linux)
- [MacOS](https://github.com/HasteHealth/HasteHealth/releases/latest/download/haste-health_macos)

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
wrk --latency -s crates/server/benchmarks/transaction.lua -t10 -c10 -d10s http://localhost:3000/w/ohio-health/zb154qm9/api/v1/fhir/r4/
```

#### M3 Macbook Air Local 10 threads Postgres 18

| Latency (percentile:time)                           | Requests per Second                                      | Concurrent connections | Benchmark                                                   |
| --------------------------------------------------- | -------------------------------------------------------- | ---------------------- | ----------------------------------------------------------- |
| 50%:1.2ms, 90%:1.8ms, 99%:3.38                      | 10344                                                    | 10                     | backend/crates/server/benchmarks/observation.lua            |
| 50%:60ms, 90%:73ms, 99%:288.6ms                     | 251 (100 resources per transaction) (25100 total writes) | 10                     | backend/crates/server/benchmarks/transaction.lua            |
| 50%:116.73ms 75%:118.39ms 90%:121.45ms 99%:246.90ms | 325 (100 reads per batch) (32500 total reads)            | 10                     | backend/crates/server/benchmarks/observation_batch_read.lua |
