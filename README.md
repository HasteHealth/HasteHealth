<div align="center">
   <img src="https://raw.githubusercontent.com/HasteHealth/HasteHealth/refs/heads/main/markdown_assets/banner.svg" style="height: 350px; width: 500px;" />
</div>

## Overview

FHIR clinical data repository built for speed.

## Running for Development

```bash
docker-compose -f docker-services-compose.yml up
cd backend
cargo run server start && cargo run worker
```

go to http://my-health_system.localhost:3001
and fill in the following credentails
username: `myuser@health.org`
password: `testing_password`

## Binaries

- [Linux](https://github.com/HasteHealth/HasteHealth/releases/latest/download/haste-health_linux)
- [MacOS](https://github.com/HasteHealth/HasteHealth/releases/latest/download/haste-health_macos)

## Docker Images

- [Server](https://github.com/HasteHealth/HasteHealth/pkgs/container/hastehealth%2Fhastehealth)
- [Admin App](https://github.com/HasteHealth/HasteHealth/pkgs/container/hastehealth%2Fadmin-app)

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

#### M3 Macbook Air Local 10 threads Postgres 16

| Latency (percentile:time)       | Requests per Second                                         | Concurrent connections | Benchmark                                        |
| ------------------------------- | ----------------------------------------------------------- | ---------------------- | ------------------------------------------------ |
| 50%:1.2ms, 90%:1.8ms, 99%:3.38  | 8058.15                                                     | 10                     | backend/crates/server/benchmarks/observation.lua |
| 50%:60ms, 90%:73ms, 99%:288.6ms | 167 (100 resources per transaction) (16,700 total requests) | 10                     | backend/crates/server/benchmarks/transaction.lua |

#### M3 Macbook Air Local 10 threads Postgres 18

| Latency (percentile:time)                           | Requests per Second                                      | Concurrent connections | Benchmark                                                   |
| --------------------------------------------------- | -------------------------------------------------------- | ---------------------- | ----------------------------------------------------------- |
| 50%:1.2ms, 90%:1.8ms, 99%:3.38                      | 10344                                                    | 10                     | backend/crates/server/benchmarks/observation.lua            |
| 50%:60ms, 90%:73ms, 99%:288.6ms                     | 251 (100 resources per transaction) (25100 total writes) | 10                     | backend/crates/server/benchmarks/transaction.lua            |
| 50%:116.73ms 75%:118.39ms 90%:121.45ms 99%:246.90ms | 325 (100 reads per batch) (32500 total reads)            | 10                     | backend/crates/server/benchmarks/observation_batch_read.lua |
