# Project Context

## Stack

### Backend

- Rust ~1.93 backend, Axum server
- ElasticSearch - search
- PostgreSQL 18 - Resource storage

## Build Commands

### Backend

- from backend/ `cargo run server start`
- from backend/ to build binary `cargo build --locked --release`

### Frontend

- compile components - from fronend/packages/components `pnpm build`
- start admin app - from frontend/packages/admin-app `pnpm start`
- build admin app - from frontend/packages/admin-app `pnpm build`

### Code generation (after changing anything in artifacts/)

- backend types - from backend/ `bash scripts/types_build.sh`
- backend operations - from backend/ `bash scripts/operation_build.sh`
- backend testscripts - from backend/ `bash scripts/testscript_build.sh`
- frontend types - from frontend/ `pnpm --filter @haste-health/fhir-types run generate`
- frontend operations - from frontend/ `pnpm --filter @haste-health/generated-ops run generate`
- artifact package .index.json (all of them) - from frontend/ `pnpm generate-indexes`.
  Every artifact package exposes a `generate-index` script and this runs them all.
  The pnpm workspace root is frontend/, so artifact scripts always run from there
  (a single package: `pnpm --filter @haste-health/haste-health.fhir.r4.core run generate-index`).
  Running `pnpm run` inside an artifacts/ directory fails - it is outside the workspace.

## Module Structure

### Backend

- backend/ is where all backend code resides
- backend/cargo.toml root of the project to start server `cargo run server start`
- crates/\*\* where various crates reside
- crates/server - server code

### Frontend

- frontend/ is where all frontend code resides
- frontend/packages/\*\* - various packages used for frontend reside here.
- frontend/packages/admin-app - Admin app for haste health.

### Artifacts

- artifacts/ holds the FHIR artifacts (profiles, search parameters, terminology,
  operations, test data) shared by the backend and the frontend. Both stacks read
  these files; there is no second copy under backend/ or frontend/.
- artifacts/r4/** and artifacts/r4b/**. One directory per FHIR package. Each is
  both an npm package (package.json + .index.json, part of the frontend pnpm
  workspace) and a source of embedded resources for the haste-artifacts crate.
- Upstream (HL7) files are never edited. A package with patches/manifest.toml (e.g.
  artifacts/r4/hl7-core/patches) builds its .min.json outputs - which the server,
  codegen and frontend load - from the upstream .json plus JSON Patch files and
  Rust rules (backend/crates/artifact-patcher). From backend/:
  build `cargo run artifacts build <manifest>`, list every change `cargo run artifacts diff <manifest>`.
  After changing a patch or rule, rebuild, then run the Code generation steps.
- artifacts/r4/hastehealth-core/definitions/haste-health/operation-frontend-only -
  OperationDefinitions used only by the TypeScript packages; excluded from the
  backend's generated ops and embedded resources on purpose.

## Conventions

- Follow conventions set forth in eslint file and cargo clippy

## Security Rules

- No hardcoded secrets
- Dependencies must not have Critical or High CVEs
- SonarQube quality gate must pass before merge

## Important Notes

- Cargo.toml and package.json files manage dependencies.
- Code in backend/crates/repository and backend/crates/fhir-search have high performance requirements.
- Editing artifacts/ affects both stacks: regenerate the Rust and TypeScript types
  and operations (see Code generation above) and the package .index.json files.
- Docker builds use the repository root as context, and the images keep artifacts/
  next to backend/ and frontend/ so the relative paths still resolve.
