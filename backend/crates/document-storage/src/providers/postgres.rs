//! One table in a PostgreSQL database.
//!
//! A document is a row of `document_storage` keyed by tenant, project and
//! name, with its body in a `bytea` column that Postgres keeps out of line. A
//! put is one upsert and a get one select, so a reader never sees part of one
//! body and part of its replacement. The body travels whole in both
//! directions, as the Binary content layer handles it anyway; one Postgres
//! value holds less than 1 GiB.
//!
//! The table comes from this crate's migrations, run with
//! [`PostgresDocumentStorage::migrate`]. Their bookkeeping table is
//! `_document_storage_migrations`, so they can share a database with another
//! application's sqlx migrations. In the Haste Health repository database the
//! migration also ties a document to its project, so deleting the project
//! deletes its documents.

use std::{
    collections::BTreeMap,
    fmt::{self, Debug},
};

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, types::Json};
use time::OffsetDateTime;

use crate::{
    error::StorageError,
    traits::{DocumentStorage, StorageFuture},
    types::{
        Document, DocumentBody, DocumentMetadata, DocumentName, DocumentScope, ListPage,
        ListRequest, StoredDocument,
    },
};

/// A body must be smaller than this: the most one Postgres value can hold.
const MAX_BODY_BYTES: u64 = 1 << 30;
/// Documents per listing page when the request names no size, and the most
/// one page holds.
const MAX_PAGE_SIZE: i64 = 1000;
const MIGRATIONS_TABLE: &str = "_document_storage_migrations";
/// SQLSTATE `undefined_table`.
const UNDEFINED_TABLE: &str = "42P01";

/// Settings for [`PostgresDocumentStorage`].
#[derive(Clone, Deserialize, Serialize)]
pub struct PostgresConfig {
    /// Database the documents are kept in. Required by
    /// [`PostgresDocumentStorage::connect`]; the Haste Health server uses its
    /// repository database when it is unset.
    #[serde(default)]
    pub database_url: Option<String>,
    /// Most connections the backend keeps open.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            database_url: None,
            max_connections: default_max_connections(),
        }
    }
}

/// Leaves the database URL out, since it may hold a password.
impl Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConfig")
            .field(
                "database_url",
                &self.database_url.as_ref().map(|_| "REDACTED"),
            )
            .field("max_connections", &self.max_connections)
            .finish()
    }
}

/// [`DocumentStorage`] over one table of a PostgreSQL database.
pub struct PostgresDocumentStorage {
    pool: PgPool,
}

/// The `document_storage` columns that make up a document's metadata, after
/// the scope the query was filtered by: name, body size (`octet_length`, an
/// `integer`), content type, metadata, `updated_at`.
type MetadataRow = (
    String,
    i32,
    Option<String>,
    Json<BTreeMap<String, String>>,
    OffsetDateTime,
);

/// What a read needs of `document_storage`: content type, metadata,
/// `updated_at`, body.
type ReadRow = (
    Option<String>,
    Json<BTreeMap<String, String>>,
    OffsetDateTime,
    Vec<u8>,
);

impl PostgresDocumentStorage {
    /// Opens a connection pool. The table need not exist yet, so this also
    /// serves to run [`Self::migrate`].
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Configuration`] when no database URL is set or
    /// the database cannot be reached.
    pub async fn connect(config: PostgresConfig) -> Result<Self, StorageError> {
        let Some(database_url) = config.database_url.as_deref() else {
            return Err(StorageError::Configuration(
                "database_url is not set".to_string(),
            ));
        };
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(database_url)
            .await
            .map_err(|error| {
                StorageError::Configuration(format!(
                    "cannot connect to the document database: {error}"
                ))
            })?;
        Ok(Self { pool })
    }

    /// Creates or updates the table.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Database`] when a migration fails.
    pub async fn migrate(&self) -> Result<(), StorageError> {
        let mut migrator = sqlx::migrate!("./pg-migrations");
        migrator.dangerous_set_table_name(MIGRATIONS_TABLE);
        migrator.run(&self.pool).await.map_err(|error| {
            StorageError::Database(format!("document storage migration failed: {error}"))
        })
    }
}

impl DocumentStorage for PostgresDocumentStorage {
    fn put<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
        document: Document,
    ) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(async move {
            let size = u64::try_from(document.body.len()).unwrap_or(u64::MAX);
            if size >= MAX_BODY_BYTES {
                return Err(StorageError::InvalidDocument(format!(
                    "body of {size} bytes is too large; one Postgres value holds less than \
                     {MAX_BODY_BYTES} bytes"
                )));
            }

            let updated_at: OffsetDateTime = sqlx::query_scalar(
                r"
                    INSERT INTO document_storage
                        (tenant, project, name, content_type, metadata, updated_at, body)
                    VALUES ($1, $2, $3, $4, $5, now(), $6)
                    ON CONFLICT (tenant, project, name) DO UPDATE SET
                        content_type = EXCLUDED.content_type,
                        metadata = EXCLUDED.metadata,
                        updated_at = EXCLUDED.updated_at,
                        body = EXCLUDED.body
                    RETURNING updated_at
                ",
            )
            .bind(scope.tenant().as_ref())
            .bind(scope.project().as_ref())
            .bind(name.as_str())
            .bind(document.content_type.as_deref())
            .bind(Json(&document.metadata))
            .bind(document.body.as_ref())
            .fetch_one(&self.pool)
            .await
            .map_err(database_error)?;

            Ok(DocumentMetadata {
                scope: scope.clone(),
                name: name.clone(),
                size,
                content_type: document.content_type,
                etag: Some(etag(size, updated_at)),
                last_modified: Some(updated_at),
                metadata: document.metadata,
            })
        })
    }

    fn head<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(async move {
            let row: Option<MetadataRow> = sqlx::query_as(
                r"
                    SELECT name, octet_length(body), content_type, metadata, updated_at
                    FROM document_storage
                    WHERE tenant = $1 AND project = $2 AND name = $3
                ",
            )
            .bind(scope.tenant().as_ref())
            .bind(scope.project().as_ref())
            .bind(name.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;
            row.map(|row| metadata_of(scope, row))
                .ok_or_else(|| StorageError::NotFound(format!("{scope}/{name}")))
        })
    }

    fn get<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, StoredDocument> {
        Box::pin(async move {
            let row: Option<ReadRow> = sqlx::query_as(
                r"
                    SELECT content_type, metadata, updated_at, body
                    FROM document_storage
                    WHERE tenant = $1 AND project = $2 AND name = $3
                ",
            )
            .bind(scope.tenant().as_ref())
            .bind(scope.project().as_ref())
            .bind(name.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;
            let (content_type, metadata, updated_at, body) =
                row.ok_or_else(|| StorageError::NotFound(format!("{scope}/{name}")))?;

            let size = u64::try_from(body.len()).unwrap_or(u64::MAX);
            Ok(StoredDocument {
                metadata: DocumentMetadata {
                    scope: scope.clone(),
                    name: name.clone(),
                    size,
                    content_type,
                    etag: Some(etag(size, updated_at)),
                    last_modified: Some(updated_at),
                    metadata: metadata.0,
                },
                body: DocumentBody::from_bytes(body),
            })
        })
    }

    fn delete<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            sqlx::query(
                "DELETE FROM document_storage WHERE tenant = $1 AND project = $2 AND name = $3",
            )
            .bind(scope.tenant().as_ref())
            .bind(scope.project().as_ref())
            .bind(name.as_str())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;
            Ok(())
        })
    }

    fn list<'a>(
        &'a self,
        scope: &'a DocumentScope,
        request: ListRequest,
    ) -> StorageFuture<'a, ListPage> {
        Box::pin(async move {
            let page_size = request.page_size.map_or(MAX_PAGE_SIZE, |size| {
                i64::from(size).clamp(1, MAX_PAGE_SIZE)
            });

            // One range of the primary key, from the token on.
            let mut rows: Vec<MetadataRow> = sqlx::query_as(
                r"
                    SELECT name, octet_length(body), content_type, metadata, updated_at
                    FROM document_storage
                    WHERE tenant = $1 AND project = $2
                      AND ($3::text IS NULL OR name > $3)
                    ORDER BY name
                    LIMIT $4
                ",
            )
            .bind(scope.tenant().as_ref())
            .bind(scope.project().as_ref())
            .bind(request.page_token)
            .bind(page_size + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

            let page_size = usize::try_from(page_size).unwrap_or_default();
            let more = rows.len() > page_size;
            rows.truncate(page_size);
            let documents = rows
                .into_iter()
                .map(|row| metadata_of(scope, row))
                .collect::<Vec<_>>();
            Ok(ListPage {
                next_page_token: if more {
                    documents
                        .last()
                        .map(|document| document.name.as_str().to_string())
                } else {
                    None
                },
                documents,
            })
        })
    }

    fn scopes(&self) -> StorageFuture<'_, Vec<DocumentScope>> {
        Box::pin(async move {
            let rows: Vec<(String, String)> = sqlx::query_as(
                r"
                    SELECT DISTINCT tenant, project
                    FROM document_storage
                    ORDER BY tenant, project
                ",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;
            rows.into_iter()
                .map(|(tenant, project)| DocumentScope::from_backend(tenant, project))
                .collect()
        })
    }
}

fn metadata_of(
    scope: &DocumentScope,
    (name, size, content_type, metadata, updated_at): MetadataRow,
) -> DocumentMetadata {
    let size = u64::try_from(size).unwrap_or_default();
    DocumentMetadata {
        scope: scope.clone(),
        name: DocumentName::from_backend(name),
        size,
        content_type,
        etag: Some(etag(size, updated_at)),
        last_modified: Some(updated_at),
        metadata: metadata.0,
    }
}

/// Changes with every put: the body's size and the moment it was stored.
fn etag(size: u64, updated_at: OffsetDateTime) -> String {
    format!("{size:x}-{:x}", updated_at.unix_timestamp_nanos())
}

fn database_error(error: sqlx::Error) -> StorageError {
    match error {
        sqlx::Error::Database(database) if database.code().as_deref() == Some(UNDEFINED_TABLE) => {
            StorageError::Configuration(
                "the document storage table is missing; run the migrations \
                 (`haste-health admin migrate repo`)"
                    .to_string(),
            )
        }
        error @ (sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::Protocol(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed) => StorageError::Transport(error.to_string()),
        error => StorageError::Database(error.to_string()),
    }
}
