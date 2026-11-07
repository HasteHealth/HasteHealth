# haste-document-storage

Object storage for document bodies, independent of where Haste Health is hosted. Callers program against the `DocumentStorage` trait (put, head, get, delete and list by key prefix); each backend sits behind a cargo feature and is built from a serde-friendly `DocumentStorageKind` with `connect`.

The Haste Health server uses it through [`haste-repository`](https://crates.io/crates/haste-repository) to keep large `Binary.data` out of the database: the content goes to document storage under `<tenant>/<project>/Binary/<id>/<versionId>` and is put back on every read.

## Install

```bash
cargo add haste-document-storage
```

## Features

| Feature      | Effect                                                                                                                      |
| ------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `filesystem` | (default) `providers::filesystem`: a directory on a local or mounted filesystem, for development, tests and single-node use |
| `gcs`        | (default) `providers::gcs`: a Google Cloud Storage bucket over the JSON API                                                 |

## Usage

```rust
use haste_document_storage::{Document, DocumentKey, DocumentStorageKind};
use haste_document_storage::providers::filesystem::FilesystemConfig;

let storage = haste_document_storage::connect(DocumentStorageKind::Filesystem(FilesystemConfig {
    root: "/var/lib/haste-health/documents".into(),
}))
.await?;

let key = DocumentKey::new("tenant/project/Binary/123/1")?;
storage
    .put(&key, Document::new(b"%PDF-1.7 ...".as_slice()).with_content_type("application/pdf"))
    .await?;

let document = storage.get(&key).await?;
let bytes = document.body.collect().await?;
```

`DocumentStorageKind` deserializes from configuration, with the backend named by a `backend` field next to its own settings:

```toml
backend = "gcs"
bucket = "my-bucket"
prefix = "documents"
credentials = { type = "service_account_key_file", path = "/secrets/key.json" }
```

Bodies are read as a stream (`DocumentBody`); `collect` gathers one into memory. Keys are validated (`DocumentKey`): non-empty, at most 1024 bytes, no control characters, no leading slash and no empty, `.` or `..` segments. Errors are `StorageError`s, which convert into a FHIR `OperationOutcome` through [`haste-fhir-operation-error`](https://crates.io/crates/haste-fhir-operation-error); `is_transient` tells a retryable failure apart, and the GCS backend retries those itself.

## Links

- [API reference](https://rust.haste.health/haste_document_storage/)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/backend/crates/document-storage)
- [Server configuration reference](https://haste.health/docs/reference/configuration) for the `document_storage` section

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
