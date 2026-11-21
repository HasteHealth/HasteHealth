# haste-repository

Resource storage for the Haste Health FHIR server: the `Repository` trait the server, the worker and the search crate program against, and its PostgreSQL implementation, `PGConnection`. It stores FHIR resources per tenant and project as an append-only version history, runs FHIR transactions, hands out the sequence numbers the search indexer follows, and keeps the server's own records: tenants, projects, users, memberships, scopes, OAuth authorization codes, MFA credentials, asynchronous jobs and failed indexing attempts.

The crate is on the path of every read and write, so it has high performance requirements. Known versions are served from an in-memory `moka` cache, writes inside a transaction are buffered and inserted in batches, and the schema (the migrations in `pg-migrations/`, embedded with `sqlx::migrate!`) is shaped for it. Changes here are measured, not assumed.

## Install

```bash
cargo add haste-repository
```

## Features

| Feature      | Effect                                                                                                                                              |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rate-limit` | Declared for the PostgreSQL implementation of `haste_rate_limit::RateLimit`. It currently gates nothing: that implementation is always compiled in. |

## Usage

```rust
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_repository::admin::Migrate;
use haste_repository::config::{PostgresRepoConfig, RepoConfig, create_repo};
use haste_repository::fhir::FHIRRepository;

let repo = create_repo(&RepoConfig::Postgres(PostgresRepoConfig {
    database_url: "postgresql://postgres:postgres@127.0.0.1/haste_health".to_string(),
    max_connections: 20,
}))
.await?;

repo.migrate().await?;

// Every FHIR call is scoped to a tenant and project and attributed to the caller's token claims.
let created = repo.create(&tenant, &project, &claims, &fhir_version, resource).await?;
let latest = repo.read_latest(&tenant, &project, &ResourceType::Patient, &resource_id).await?;

// A transaction is a second connection value: writes stay invisible until commit.
let tx = repo.transaction(true).await?;
tx.update(&tenant, &project, &claims, &fhir_version, changed, &id).await?;
tx.commit().await?;
```

`RepoConfig` is the `[repo]` section of the server configuration (`HASTE_REPO.database_url` and friends). `transaction(register)` opens a database transaction; `true` registers it with the sequence guard so the indexer never skips a write that is still committing. `claims` is a `haste_jwt::claims::UserTokenClaims` and `fhir_version` a `haste_jwt::SupportedFHIRVersions`.

## API

| Trait or module                                        | Covers                                                                                                                                                               |
| ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fhir::FHIRRepository`                                 | `create`, `update`, `delete`, `read_latest`, `read_latest_many`, `read_by_version_ids`, `history`, and `transaction`, `commit`, `rollback`                           |
| `admin::TenantModelAdmin`, `admin::ProjectModelAdmin`  | Create, read, update, delete and search for tenants, users, projects, authorization codes and MFA credentials (tenant scope), memberships and scopes (project scope) |
| `admin::Login`, `admin::Migrate`, `admin::SystemAdmin` | Email and password or OIDC login, schema migrations, cross-tenant user search                                                                                        |
| `sequence::ResourceSequential`                         | `max_safe_sequence` and `get_sequence`: the ordered, gap-free feed of writes the search indexer consumes                                                             |
| `async_job::AsyncJobProvider`                          | Queue, claim, lease and complete jobs for the FHIR Asynchronous Request Pattern                                                                                      |
| `failed_indexing::FailedIndexingProvider`              | Record and list resources that failed search indexing                                                                                                                |
| `config::RepoConfig`, `config::create_repo`            | The storage configuration and the pool it opens                                                                                                                      |
| `pg::PGConnection`, `pg::StoreError`                   | The PostgreSQL implementation (`Pool` or `Transaction`) and its error type                                                                                           |

The integration tests need a reachable PostgreSQL instance: `DATABASE_URL=postgresql://postgres:postgres@127.0.0.1/haste_health cargo test -p haste-repository`.

## Links

- [API reference](https://rust.haste.health/haste_repository/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/repository)
- [Rust SDK overview](https://haste.health/docs/api/sdks/rust)
- [Platform architecture](https://haste.health/docs/core_concepts/platform_architecture) and the [server configuration reference](https://haste.health/docs/reference/configuration) for the `repo` section
- Related crates: [haste-fhir-model](https://crates.io/crates/haste-fhir-model), [haste-jwt](https://crates.io/crates/haste-jwt), [haste-rate-limit](https://crates.io/crates/haste-rate-limit), [haste-subscription](https://crates.io/crates/haste-subscription)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
