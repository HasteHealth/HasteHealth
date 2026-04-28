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

## Module Structure
### Backend
- backend/ is where all backend code resides
- backend/cargo.toml root of the project to start server `cargo run server start`
- crates/** where various crates reside
- crates/server - server code
### Frontend
- frontend/ is where all frontend code resides
- frontend/packages/** - various packages used for frontend reside here.
- frontend/packages/admin-app - Admin app for haste health.


## Conventions
- Follow conventions set forth in eslint file and cargo clippy

## Security Rules
- No hardcoded secrets
- Dependencies must not have Critical or High CVEs
- SonarQube quality gate must pass before merge

## Important Notes
- Cargo.toml and package.json files manage dependencies.
- Code in backend/crates/repository and backend/crates/fhir-search have high performance requirements.