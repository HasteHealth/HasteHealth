//! Tables in a PostgreSQL database.
//!
//! A document is a row of `document_storage` keyed by tenant, project and
//! name (size, content type, metadata), and its body is spread over rows of
//! `document_storage_chunks`, at most 1 MiB each, so a reader can stream it
//! and no single value comes near Postgres's 1 GB limit. Every put takes a new
//! generation from a sequence and stores its chunks under it; a get reads the
//! generation first and then only that generation's chunks, the same way the
//! GCS backend pins an object generation.
//!
//! The tables come from this crate's migrations, run with
//! [`PostgresDocumentStorage::migrate`]. Their bookkeeping table is
//! `_document_storage_migrations`, so they can share a database with another
//! application's sqlx migrations. In the Haste Health repository database the
//! migration also ties a document to its project, so deleting the project
//! deletes its documents.

use std::{
    collections::BTreeMap,
    fmt::{self, Debug},
};

use bytes::Bytes;
use futures_util::stream;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, types::Json};
use time::OffsetDateTime;

use crate::{
    error::StorageError,
    traits::{DocumentStorage, StorageFuture},
    types::{
        Document, DocumentBody, DocumentMetadata, DocumentName, DocumentScope, ListPage,
        ListRequest, StoredDocument, page_token, parse_page_token,
    },
};

/// Largest chunk of a body stored in one row.
const CHUNK_SIZE: usize = 1024 * 1024;
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

/// [`DocumentStorage`] over two tables of a PostgreSQL database.
pub struct PostgresDocumentStorage {
    pool: PgPool,
}

/// The `document_storage` columns that make up a document's metadata.
type MetadataRow = (
    String,
    String,
    String,
    i64,
    i64,
    Option<String>,
    Json<BTreeMap<String, String>>,
    OffsetDateTime,
);

/// What a read needs of `document_storage`, with the first chunk of the body.
type ReadRow = (
    i64,
    i64,
    i32,
    Option<String>,
    Json<BTreeMap<String, String>>,
    OffsetDateTime,
    Option<Vec<u8>>,
);

impl PostgresDocumentStorage {
    /// Opens a connection pool. The tables need not exist yet, so this also
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

    /// Creates or updates the tables.
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
            let too_large = || StorageError::InvalidDocument("body is too large".to_string());
            let size = i64::try_from(document.body.len()).map_err(|_| too_large())?;
            let chunks =
                i32::try_from(document.body.len().div_ceil(CHUNK_SIZE)).map_err(|_| too_large())?;
            let (tenant, project) = (scope.tenant().as_ref(), scope.project().as_ref());

            let mut tx = self.pool.begin().await.map_err(database_error)?;
            // Takes the row lock first, so concurrent puts of one document
            // run one after the other. `xmax = 0` tells a new row from a
            // replaced one.
            let (generation, inserted, updated_at): (i64, bool, OffsetDateTime) = sqlx::query_as(
                r"
                    INSERT INTO document_storage
                        (tenant, project, name, generation, size, chunks, content_type,
                         metadata, updated_at)
                    VALUES ($1, $2, $3, nextval('document_storage_generations'), $4, $5, $6,
                            $7, now())
                    ON CONFLICT (tenant, project, name) DO UPDATE SET
                        generation = EXCLUDED.generation,
                        size = EXCLUDED.size,
                        chunks = EXCLUDED.chunks,
                        content_type = EXCLUDED.content_type,
                        metadata = EXCLUDED.metadata,
                        updated_at = EXCLUDED.updated_at
                    RETURNING generation, (xmax = 0), updated_at
                ",
            )
            .bind(tenant)
            .bind(project)
            .bind(name.as_str())
            .bind(size)
            .bind(chunks)
            .bind(document.content_type.as_deref())
            .bind(Json(&document.metadata))
            .fetch_one(&mut *tx)
            .await
            .map_err(database_error)?;

            for (index, chunk) in (0..chunks).zip(document.body.chunks(CHUNK_SIZE)) {
                sqlx::query(
                    r"
                        INSERT INTO document_storage_chunks
                            (tenant, project, name, generation, chunk, data)
                        VALUES ($1, $2, $3, $4, $5, $6)
                    ",
                )
                .bind(tenant)
                .bind(project)
                .bind(name.as_str())
                .bind(generation)
                .bind(index)
                .bind(chunk)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
            }

            if !inserted {
                sqlx::query(
                    r"
                        DELETE FROM document_storage_chunks
                        WHERE tenant = $1 AND project = $2 AND name = $3 AND generation <> $4
                    ",
                )
                .bind(tenant)
                .bind(project)
                .bind(name.as_str())
                .bind(generation)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
            }
            tx.commit().await.map_err(database_error)?;

            Ok(DocumentMetadata {
                scope: scope.clone(),
                name: name.clone(),
                size: size.unsigned_abs(),
                content_type: document.content_type,
                etag: Some(format!("{generation:x}")),
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
                    SELECT tenant, project, name, generation, size, content_type, metadata,
                           updated_at
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
            row.map(metadata_of)
                .ok_or_else(|| StorageError::NotFound(format!("{scope}/{name}")))
        })
    }

    fn get<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, StoredDocument> {
        Box::pin(async move {
            let subject = format!("{scope}/{name}");
            // The metadata and the first chunk in one round trip, which is
            // the whole body for most documents.
            let row: Option<ReadRow> = sqlx::query_as(
                r"
                    SELECT d.generation, d.size, d.chunks, d.content_type, d.metadata,
                           d.updated_at, c.data
                    FROM document_storage d
                    LEFT JOIN document_storage_chunks c
                        ON c.tenant = d.tenant AND c.project = d.project AND c.name = d.name
                       AND c.generation = d.generation AND c.chunk = 0
                    WHERE d.tenant = $1 AND d.project = $2 AND d.name = $3
                ",
            )
            .bind(scope.tenant().as_ref())
            .bind(scope.project().as_ref())
            .bind(name.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

            let Some((generation, size, chunks, content_type, metadata, updated_at, first)) = row
            else {
                return Err(StorageError::NotFound(subject));
            };
            if chunks > 0 && first.is_none() {
                // Replaced between the two reads of the join.
                return Err(StorageError::NotFound(subject));
            }

            let state = ChunkReader {
                pool: self.pool.clone(),
                scope: scope.clone(),
                name: name.clone(),
                generation,
                next: 1,
                chunks,
                first: first.map(Bytes::from),
            };
            let body = stream::try_unfold(state, |mut state| async move {
                if let Some(first) = state.first.take() {
                    return Ok(Some((first, state)));
                }
                if state.next >= state.chunks {
                    return Ok(None);
                }
                let data: Option<Vec<u8>> = sqlx::query_scalar(
                    r"
                        SELECT data FROM document_storage_chunks
                        WHERE tenant = $1 AND project = $2 AND name = $3
                          AND generation = $4 AND chunk = $5
                    ",
                )
                .bind(state.scope.tenant().as_ref())
                .bind(state.scope.project().as_ref())
                .bind(state.name.as_str())
                .bind(state.generation)
                .bind(state.next)
                .fetch_optional(&state.pool)
                .await
                .map_err(database_error)?;
                // A missing chunk means the document was replaced or deleted
                // while it was being read.
                let data = data.ok_or_else(|| {
                    StorageError::NotFound(format!("{}/{}", state.scope, state.name))
                })?;
                state.next += 1;
                Ok(Some((Bytes::from(data), state)))
            });

            Ok(StoredDocument {
                metadata: DocumentMetadata {
                    scope: scope.clone(),
                    name: name.clone(),
                    size: u64::try_from(size).unwrap_or_default(),
                    content_type,
                    etag: Some(format!("{generation:x}")),
                    last_modified: Some(updated_at),
                    metadata: metadata.0,
                },
                body: DocumentBody::from_stream(body),
            })
        })
    }

    fn delete<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            // The chunks go with the row through their foreign key.
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

    fn list(&self, request: ListRequest) -> StorageFuture<'_, ListPage> {
        Box::pin(async move {
            let page_size = request.page_size.map_or(MAX_PAGE_SIZE, |size| {
                i64::from(size).clamp(1, MAX_PAGE_SIZE)
            });
            let scope = request.scope.as_ref();
            let after = request
                .page_token
                .as_deref()
                .map(parse_page_token)
                .transpose()?;

            // The row comparison walks the primary key from the token on.
            let mut rows: Vec<MetadataRow> = sqlx::query_as(
                r"
                    SELECT tenant, project, name, generation, size, content_type, metadata,
                           updated_at
                    FROM document_storage
                    WHERE ($1::text IS NULL OR (tenant = $1 AND project = $2))
                      AND ($3::text IS NULL OR (tenant, project, name) > ($3, $4, $5))
                    ORDER BY tenant, project, name
                    LIMIT $6
                ",
            )
            .bind(scope.map(|scope| scope.tenant().as_ref()))
            .bind(scope.map(|scope| scope.project().as_ref()))
            .bind(after.as_ref().map(|(tenant, _, _)| tenant.as_str()))
            .bind(after.as_ref().map(|(_, project, _)| project.as_str()))
            .bind(after.as_ref().map(|(_, _, name)| name.as_str()))
            .bind(page_size + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

            let page_size = usize::try_from(page_size).unwrap_or_default();
            let more = rows.len() > page_size;
            rows.truncate(page_size);
            let documents = rows.into_iter().map(metadata_of).collect::<Vec<_>>();
            Ok(ListPage {
                next_page_token: if more {
                    documents
                        .last()
                        .map(|document| page_token(&document.scope, document.name.as_str()))
                } else {
                    None
                },
                documents,
            })
        })
    }
}

/// Where a streaming read of a body stands.
struct ChunkReader {
    pool: PgPool,
    scope: DocumentScope,
    name: DocumentName,
    generation: i64,
    next: i32,
    chunks: i32,
    first: Option<Bytes>,
}

fn metadata_of(
    (tenant, project, name, generation, size, content_type, metadata, updated_at): MetadataRow,
) -> DocumentMetadata {
    DocumentMetadata {
        scope: DocumentScope::from_backend(tenant, project),
        name: DocumentName::from_backend(name),
        size: u64::try_from(size).unwrap_or_default(),
        content_type,
        etag: Some(format!("{generation:x}")),
        last_modified: Some(updated_at),
        metadata: metadata.0,
    }
}

fn database_error(error: sqlx::Error) -> StorageError {
    match error {
        sqlx::Error::Database(database) if database.code().as_deref() == Some(UNDEFINED_TABLE) => {
            StorageError::Configuration(
                "the document storage tables are missing; run the migrations \
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
