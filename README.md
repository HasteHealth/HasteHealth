# Oxidized Health

## Running for Development

```bash
cd backend
cargo run server migrate all
cargo run server migrate artifacts
cargo run server start && cargo run worker
```

## Repository Structure

```text
# Repository structure

/ (project root)
├── README.md
├── rust-toolchain.toml
├── cargo.toml
├── backend/
    └── crates/
        ├── access-control/
        ├── artifacts/
        ├── codegen/
        ├── fhir-client/
        ├── fhir-generated-ops/
        ├── fhir-model/
        ├── fhir-operation-error/
        ├── fhir-ops/
        ├── fhir-operation-derive/
        ├── fhir-search/
        ├── fhir-serialization-json/
        ├── fhir-serialization-json-derive/
        ├── fhir-terminology/
        ├── fhirpath/
        ├── indexing-worker/
        ├── jwt/
        ├── macro-loads/
        ├── reflect/
        ├── reflect-derive/
        ├── repository/
        └── server/
├── src/                           # Entry point for backend
└── scripts/
    ├── operation_build.sh.        # Generate OperationDefinition code
    └── types_build.sh.            # Generate FHIR rust types
frontend/
    └── packages/
        ├── admin-app
        ├── artifacts
        ├── cli
        ├── client
        ├── codegen
        ├── components
        ├── fhir-patch-building
        ├── fhir-pointer
        ├── fhir-types
        ├── fhir-validation
        ├── fhirpath
        ├── generated-ops
        ├── hl7v2-parsing
        ├── jwt
        ├── koa-multipart-form
        ├── lang-fp-codemirror
        ├── meta-value
        ├── operation-execution
        ├── operation-outcomes
        ├── performance-testing
        ├── smart-launch
        ├── testscript-runner
        ├── x-fhir-query

├── .github/
│ └── workflows/ # CI/CD workflows
└── LICENSE
```

## Compiled Binaries

- [Linux](https://github.com/OxidizedHealth/oxidized-health/releases/latest/download/oxidized-health_linux)
- [MacOS](https://github.com/OxidizedHealth/oxidized-health/releases/latest/download/oxidized-health_macos)
