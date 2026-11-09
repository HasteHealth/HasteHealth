# haste-document-storage

Object storage for document bodies, independent of where Haste Health is hosted. Callers program against the `DocumentStorage` trait (put, head, get, delete and list, each within one tenant and project; `scopes` enumerates the tenants and projects the store holds, for maintenance); each backend sits behind a cargo feature and is built from a serde-friendly `DocumentStorageKind` with `connect`.

Every call names the tenant and project the document belongs to (a `DocumentScope`, built from `haste-jwt`'s `TenantId` and `ProjectId`) and the document's name within them (a `DocumentName`), so a caller can only ever reach its own project's documents. Backends keep the scope apart from the name: columns in Postgres, a directory per tenant and project on the filesystem, the leading segments of the object name in a bucket.

The Haste Health server uses it to keep large `Binary.data` out of the resource rows: a layer of its FHIR pipeline moves the content to document storage as `Binary/<document id>` in the resource's tenant and project on the way in and puts it back on the way out, so the repository only ever stores row-sized Binaries. By default that storage is the `postgres` backend in the repository database.

## Install

```bash
cargo add haste-document-storage
```

## Features

| Feature      | Effect                                                                                                                      |
| ------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `filesystem` | (default) `providers::filesystem`: a directory on a local or mounted filesystem, for development, tests and single-node use |
| `gcs`        | (default) `providers::gcs`: a Google Cloud Storage bucket over the JSON API                                                 |
| `postgres`   | (default) `providers::postgres`: two tables in a PostgreSQL database, the body in rows of at most 1 MiB                     |

## Usage

```rust
use haste_document_storage::{Document, DocumentName, DocumentScope, DocumentStorageKind};
use haste_document_storage::providers::filesystem::FilesystemConfig;
use haste_jwt::{ProjectId, TenantId};

let storage = haste_document_storage::connect(DocumentStorageKind::Filesystem(FilesystemConfig {
    root: "/var/lib/haste-health/documents".into(),
}))
.await?;

let scope = DocumentScope::new(&TenantId::new("acme".into())?, &ProjectId::new("clinic".into())?);
let name = DocumentName::new("Binary/123")?;
storage
    .put(&scope, &name, Document::new(b"%PDF-1.7 ...".as_slice()).with_content_type("application/pdf"))
    .await?;

let document = storage.get(&scope, &name).await?;
let bytes = document.body.collect().await?;
```

`DocumentStorageKind` deserializes from configuration, with the backend named by a `backend` field next to its own settings:

```toml
backend = "gcs"
bucket = "my-bucket"
prefix = "documents"
credentials = { type = "service_account_key_file", path = "/secrets/key.json" }
```

The `postgres` backend needs its tables before first use: `PostgresDocumentStorage::migrate` creates them, recording its migrations in `_document_storage_migrations` so it can share a database with another application's sqlx migrations. In a database that also holds Haste Health's `projects` table, the migration ties every document to its project, so deleting the project deletes its documents.

Bodies are read as a stream (`DocumentBody`); `collect` gathers one into memory. Names are validated (`DocumentName`): non-empty, at most 1024 bytes, no control characters, no leading slash and no empty, `.` or `..` segments. Errors are `StorageError`s, which convert into a FHIR `OperationOutcome` through [`haste-fhir-operation-error`](https://crates.io/crates/haste-fhir-operation-error); `is_transient` tells a retryable failure apart, and the GCS backend retries those itself.

## Links

- [API reference](https://rust.haste.health/haste_document_storage/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/document-storage)
- [Server configuration reference](https://haste.health/docs/reference/configuration) for the `document_storage` section

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
