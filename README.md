# Haste Health

## Running for Development

```bash
cd backend
cargo run server migrate all
cargo run server migrate artifacts
cargo run server start && cargo run worker
```

## Compiled Binaries

- [Linux](https://github.com/HasteHealth/haste-health/releases/latest/download/haste-health_linux)
- [MacOS](https://github.com/HasteHealth/haste-health/releases/latest/download/haste-health_macos)

## Repository Structure

```
├── LICENSE
├── README.md
├── backend # Backend entry point see above for commands
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── certifications
│   ├── crates
│   │   ├── access-control
│   │   ├── artifacts
│   │   ├── codegen
│   │   ├── config
│   │   ├── fhir-client
│   │   ├── fhir-generated-ops
│   │   ├── fhir-model
│   │   ├── fhir-operation-error
│   │   ├── fhir-operation-error-derive
│   │   ├── fhir-ops
│   │   ├── fhir-ops-derive
│   │   ├── fhir-search
│   │   ├── fhir-serialization-json
│   │   ├── fhir-serialization-json-derive
│   │   ├── fhir-terminology
│   │   ├── fhirpath
│   │   ├── indexing-worker
│   │   ├── jwt
│   │   ├── macro-loads
│   │   ├── reflect
│   │   ├── reflect-derive
│   │   ├── repository
│   │   └── server             # FHIR server.
│   ├── documentation          # Documentation site.
│   │   ├── book.toml
│   │   └── src
│   ├── rust-toolchain.toml
│   ├── scripts
│   │   ├── operation_build.sh # Generates code for parsing OperationDefinition parameters using codegen crate.
│   │   └── types_build.sh     # Generates rust types using FHIR StructureDefinition resources.
│   └── src
│       ├── commands
│       └── main.rs
└── frontend
    ├── README.md
    ├── artifacts
    │   ├── r4
    │   └── r4b
    ├── config
    │   ├── base.tsconfig.json
    │   └── jest.base.config.js
    ├── package.json
    ├── packages
    │   ├── admin-app
    │   ├── artifacts
    │   ├── cli
    │   ├── client
    │   ├── codegen
    │   ├── components
    │   ├── fhir-patch-building
    │   ├── fhir-pointer
    │   ├── fhir-types
    │   ├── fhir-validation
    │   ├── fhirpath
    │   ├── generated-ops
    │   ├── hl7v2-parsing
    │   ├── jwt
    │   ├── koa-multipart-form
    │   ├── lang-fp-codemirror
    │   ├── meta-value
    │   ├── operation-execution
    │   ├── operation-outcomes
    │   ├── performance-testing
    │   ├── smart-launch
    │   ├── testscript-runner
    │   └── x-fhir-query
    └── yarn.lock
```

## Performance

Using `wrk` for performance testing.

### Example

```bash
wrk --latency -s crates/server/benchmarks/transaction.lua -t10 -c10 -d10s http://localhost:3000/w/ohio-health/api/v1/zb154qm9/fhir/r4/
```

#### M3 Macbook Air Local 10 threads Postgres 16

| Latency (percentile:time)       | Requests per Second                                         | Concurrent connections | Benchmark                                        |
| ------------------------------- | ----------------------------------------------------------- | ---------------------- | ------------------------------------------------ |
| 50%:1.2ms, 90%:1.8ms, 99%:3.38  | 8058.15                                                     | 10                     | backend/crates/server/benchmarks/observation.lua |
| 50%:60ms, 90%:73ms, 99%:288.6ms | 167 (100 resources per transaction) (16,700 total requests) | 10                     | backend/crates/server/benchmarks/transaction.lua |

#### M3 Macbook Air Local 10 threads Postgres 18

| Latency (percentile:time)       | Requests per Second                                        | Concurrent connections | Benchmark                                        |
| ------------------------------- | ---------------------------------------------------------- | ---------------------- | ------------------------------------------------ |
| 50%:1.2ms, 90%:1.8ms, 99%:3.38  | 9401                                                       | 10                     | backend/crates/server/benchmarks/observation.lua |
| 50%:60ms, 90%:73ms, 99%:288.6ms | 201 (100 resources per transaction) (20100 total requests) | 10                     | backend/crates/server/benchmarks/transaction.lua |
