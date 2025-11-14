# Oxidized Health

## Running for Development

```bash
cd backend
cargo run server migrate all
cargo run server migrate artifacts
cargo run server start && cargo run worker
```

## Compiled Binaries

- [Linux](https://github.com/OxidizedHealth/oxidized-health/releases/latest/download/oxidized-health_linux)
- [MacOS](https://github.com/OxidizedHealth/oxidized-health/releases/latest/download/oxidized-health_macos)

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
