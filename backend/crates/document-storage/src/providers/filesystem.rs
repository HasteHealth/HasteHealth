//! A directory on a local or mounted filesystem.
//!
//! A document lives at `<root>/<tenant>/<project>/<name>`: a directory per
//! tenant, one per project inside it, and the name's segments below that.
//! Tenant and project ids are letters, digits and hyphens, so they are used
//! as they are. Every name segment is percent-encoded (everything but ASCII
//! letters, digits, `-`, `_` and `~`, so `.` as well); the last one names two
//! files: `<segment>.data` holds the body and `<segment>.json` the content
//! type and metadata. Because `.` is always encoded, those names never clash
//! with a directory made for a longer name, and the temporary files a put
//! writes (whose names start with a dot) never show up in a listing.
//!
//! Meant for development, tests and single-node deployments. Names that differ
//! only in letter case collide on a case-insensitive filesystem.

use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::UNIX_EPOCH,
};

use bytes::BytesMut;
use futures_util::stream;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::{
    fs,
    io::{AsyncReadExt as _, AsyncWriteExt as _},
};

use crate::{
    error::StorageError,
    traits::{DocumentStorage, StorageFuture},
    types::{
        Document, DocumentBody, DocumentMetadata, DocumentName, DocumentScope, ListPage,
        ListRequest, StoredDocument,
    },
};

const DATA_SUFFIX: &str = ".data";
const METADATA_SUFFIX: &str = ".json";
/// Bytes read from disk per body chunk.
const CHUNK_SIZE: usize = 64 * 1024;
/// Documents per listing page when the request names no size, and the most
/// one page holds.
const MAX_PAGE_SIZE: usize = 1000;

const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'~');

/// Settings for [`FilesystemDocumentStorage`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesystemConfig {
    /// Directory the documents are kept under. Created when missing.
    pub root: PathBuf,
}

/// [`DocumentStorage`] over one directory.
pub struct FilesystemDocumentStorage {
    root: PathBuf,
}

/// What `<segment>.json` holds.
#[derive(Default, Deserialize, Serialize)]
struct StoredMetadata {
    content_type: Option<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

impl FilesystemDocumentStorage {
    /// Creates the root directory if needed.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Configuration`] when the root is empty or
    /// cannot be created.
    pub async fn connect(config: FilesystemConfig) -> Result<Self, StorageError> {
        if config.root.as_os_str().is_empty() {
            return Err(StorageError::Configuration("root is empty".to_string()));
        }
        let unusable = |error: io::Error| {
            StorageError::Configuration(format!(
                "cannot use '{}' as the document root: {error}",
                config.root.display()
            ))
        };
        fs::create_dir_all(&config.root).await.map_err(unusable)?;
        let root = fs::canonicalize(&config.root).await.map_err(unusable)?;
        Ok(Self { root })
    }

    /// The directory a scope's documents live under.
    fn scope_directory(&self, scope: &DocumentScope) -> PathBuf {
        let mut directory = self.root.clone();
        directory.push(scope.tenant().as_ref());
        directory.push(scope.project().as_ref());
        directory
    }

    /// The directory holding a document's files, and the stem of their names.
    fn location(&self, scope: &DocumentScope, name: &str) -> (PathBuf, String) {
        let mut segments = name.split('/');
        let last = segments.next_back().unwrap_or_default();
        let mut directory = self.scope_directory(scope);
        for segment in segments {
            directory.push(encode(segment));
        }
        (directory, encode(last))
    }

    fn data_path(&self, scope: &DocumentScope, name: &DocumentName) -> PathBuf {
        let (directory, stem) = self.location(scope, name.as_str());
        directory.join(format!("{stem}{DATA_SUFFIX}"))
    }

    async fn metadata_of(
        &self,
        scope: &DocumentScope,
        name: &DocumentName,
    ) -> Result<DocumentMetadata, StorageError> {
        let (directory, stem) = self.location(scope, name.as_str());
        let file = fs::metadata(directory.join(format!("{stem}{DATA_SUFFIX}")))
            .await
            .map_err(|error| io_error(scope, name, &error))?;
        let stored = match fs::read(directory.join(format!("{stem}{METADATA_SUFFIX}"))).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| {
                StorageError::Io(format!(
                    "metadata of '{scope}/{name}' is unreadable: {error}"
                ))
            })?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => StoredMetadata::default(),
            Err(error) => return Err(io_error(scope, name, &error)),
        };
        let modified = file.modified().ok();
        let modified_nanos = modified
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();

        Ok(DocumentMetadata {
            scope: scope.clone(),
            name: name.clone(),
            size: file.len(),
            content_type: stored.content_type,
            etag: Some(format!("{:x}-{modified_nanos:x}", file.len())),
            last_modified: modified.map(OffsetDateTime::from),
            metadata: stored.metadata,
        })
    }

    async fn write_files(
        &self,
        scope: &DocumentScope,
        name: &DocumentName,
        document: &Document,
        stored: &[u8],
    ) -> io::Result<()> {
        let (directory, stem) = self.location(scope, name.as_str());
        fs::create_dir_all(&directory).await?;
        write_atomically(
            &directory,
            &directory.join(format!("{stem}{METADATA_SUFFIX}")),
            stored,
        )
        .await?;
        write_atomically(
            &directory,
            &directory.join(format!("{stem}{DATA_SUFFIX}")),
            &document.body,
        )
        .await
    }

    /// Removes `directory` and then its parents while they are empty,
    /// stopping at the root.
    async fn prune(&self, mut directory: PathBuf) {
        while directory != self.root && directory.starts_with(&self.root) {
            if fs::remove_dir(&directory).await.is_err() {
                return;
            }
            if !directory.pop() {
                return;
            }
        }
    }

    /// The subdirectories of `directory`, with their names.
    async fn subdirectories(
        &self,
        directory: &Path,
    ) -> Result<Vec<(PathBuf, String)>, StorageError> {
        let mut found = Vec::new();
        let Some(mut entries) = read_dir(directory).await? else {
            return Ok(found);
        };
        while let Some(entry) = next_entry(&mut entries, directory).await? {
            if file_type(&entry, directory).await?.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                found.push((entry.path(), name.to_string()));
            }
        }
        Ok(found)
    }

    /// The name of every stored document of `scope`, unsorted.
    async fn names(&self, scope: &DocumentScope) -> Result<Vec<String>, StorageError> {
        let mut found = Vec::new();
        let mut pending = vec![(self.scope_directory(scope), String::new())];
        while let Some((directory, name_prefix)) = pending.pop() {
            let Some(mut entries) = read_dir(&directory).await? else {
                continue;
            };
            while let Some(entry) = next_entry(&mut entries, &directory).await? {
                let file_name = entry.file_name();
                let Some(file_name) = file_name.to_str() else {
                    continue;
                };
                if file_type(&entry, &directory).await?.is_dir() {
                    if let Some(segment) = decode(file_name) {
                        pending.push((entry.path(), format!("{name_prefix}{segment}/")));
                    }
                } else if let Some(segment) = file_name.strip_suffix(DATA_SUFFIX).and_then(decode) {
                    found.push(format!("{name_prefix}{segment}"));
                }
            }
        }
        Ok(found)
    }
}

impl DocumentStorage for FilesystemDocumentStorage {
    fn put<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
        document: Document,
    ) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(async move {
            let stored = serde_json::to_vec(&StoredMetadata {
                content_type: document.content_type.clone(),
                metadata: document.metadata.clone(),
            })
            .map_err(|error| {
                StorageError::InvalidDocument(format!("metadata is not serializable: {error}"))
            })?;

            // A delete of a neighbouring document may remove the directory
            // between creating it and writing into it; one more try recreates
            // it.
            let mut result = self.write_files(scope, name, &document, &stored).await;
            if matches!(&result, Err(error) if error.kind() == io::ErrorKind::NotFound) {
                result = self.write_files(scope, name, &document, &stored).await;
            }
            result.map_err(|error| io_error(scope, name, &error))?;

            self.metadata_of(scope, name).await
        })
    }

    fn head<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(self.metadata_of(scope, name))
    }

    fn get<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, StoredDocument> {
        Box::pin(async move {
            let metadata = self.metadata_of(scope, name).await?;
            let file = fs::File::open(self.data_path(scope, name))
                .await
                .map_err(|error| io_error(scope, name, &error))?;
            let subject = format!("{scope}/{name}");
            let body = stream::try_unfold(file, move |mut file| {
                let subject = subject.clone();
                async move {
                    let mut chunk = BytesMut::with_capacity(CHUNK_SIZE);
                    let read = file.read_buf(&mut chunk).await.map_err(|error| {
                        StorageError::Io(format!("reading '{subject}' failed: {error}"))
                    })?;
                    Ok((read > 0).then(|| (chunk.freeze(), file)))
                }
            });
            Ok(StoredDocument {
                metadata,
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
            let (directory, stem) = self.location(scope, name.as_str());
            for suffix in [DATA_SUFFIX, METADATA_SUFFIX] {
                match fs::remove_file(directory.join(format!("{stem}{suffix}"))).await {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(io_error(scope, name, &error)),
                }
            }
            self.prune(directory).await;
            Ok(())
        })
    }

    fn list<'a>(
        &'a self,
        scope: &'a DocumentScope,
        request: ListRequest,
    ) -> StorageFuture<'a, ListPage> {
        Box::pin(async move {
            let mut names = self.names(scope).await?;
            names.sort_unstable();
            if let Some(token) = &request.page_token {
                names.retain(|name| name > token);
            }

            let page_size = request
                .page_size
                .and_then(|size| usize::try_from(size).ok())
                .filter(|size| *size > 0)
                .map_or(MAX_PAGE_SIZE, |size| size.min(MAX_PAGE_SIZE));
            let more = names.len() > page_size;
            names.truncate(page_size);

            let mut documents = Vec::with_capacity(names.len());
            for name in &names {
                match self
                    .metadata_of(scope, &DocumentName::from_backend(name.clone()))
                    .await
                {
                    Ok(metadata) => documents.push(metadata),
                    // Deleted since the directory was read.
                    Err(StorageError::NotFound(_)) => {}
                    Err(error) => return Err(error),
                }
            }

            Ok(ListPage {
                documents,
                next_page_token: if more { names.last().cloned() } else { None },
            })
        })
    }

    /// The root's subdirectories are tenants, theirs are projects. Anything
    /// else in those two levels is not this backend's and fails the call.
    fn scopes(&self) -> StorageFuture<'_, Vec<DocumentScope>> {
        Box::pin(async move {
            let mut scopes = Vec::new();
            for (tenant_directory, tenant) in self.subdirectories(&self.root).await? {
                for (_, project) in self.subdirectories(&tenant_directory).await? {
                    scopes.push(DocumentScope::from_backend(tenant.clone(), project)?);
                }
            }
            Ok(scopes)
        })
    }
}

fn encode(segment: &str) -> String {
    utf8_percent_encode(segment, ENCODE_SET).to_string()
}

/// The segment a file or directory name stands for, or `None` for a name
/// this backend did not write.
fn decode(name: &str) -> Option<String> {
    if name.is_empty() || name.contains('.') {
        return None;
    }
    percent_decode_str(name)
        .decode_utf8()
        .ok()
        .map(std::borrow::Cow::into_owned)
}

/// Opens `directory` for listing; `None` when it does not exist.
async fn read_dir(directory: &Path) -> Result<Option<fs::ReadDir>, StorageError> {
    match fs::read_dir(directory).await {
        Ok(entries) => Ok(Some(entries)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(list_error(directory, &error)),
    }
}

async fn next_entry(
    entries: &mut fs::ReadDir,
    directory: &Path,
) -> Result<Option<fs::DirEntry>, StorageError> {
    entries
        .next_entry()
        .await
        .map_err(|error| list_error(directory, &error))
}

async fn file_type(
    entry: &fs::DirEntry,
    directory: &Path,
) -> Result<std::fs::FileType, StorageError> {
    entry
        .file_type()
        .await
        .map_err(|error| list_error(directory, &error))
}

fn list_error(directory: &Path, error: &io::Error) -> StorageError {
    StorageError::Io(format!("cannot list '{}': {error}", directory.display()))
}

/// Writes `contents` to a temporary file in `directory`, flushes it to disk
/// and renames it over `path`, so readers see the old file or the new one,
/// never part of one.
async fn write_atomically(directory: &Path, path: &Path, contents: &[u8]) -> io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temporary = directory.join(format!(
        ".{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let result = async {
        let mut file = fs::File::create(&temporary).await?;
        file.write_all(contents).await?;
        file.sync_all().await?;
        fs::rename(&temporary, path).await
    }
    .await;
    if result.is_err() {
        let _ = fs::remove_file(&temporary).await;
    }
    result
}

fn io_error(scope: &DocumentScope, name: &DocumentName, error: &io::Error) -> StorageError {
    match error.kind() {
        io::ErrorKind::NotFound => StorageError::NotFound(format!("{scope}/{name}")),
        io::ErrorKind::PermissionDenied => {
            StorageError::Forbidden(format!("'{scope}/{name}': {error}"))
        }
        _ => StorageError::Io(format!("'{scope}/{name}': {error}")),
    }
}
