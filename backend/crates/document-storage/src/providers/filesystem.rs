//! A directory on a local or mounted filesystem.
//!
//! A key becomes a path under the root. Every segment is percent-encoded
//! (everything but ASCII letters, digits, `-`, `_` and `~`, so `.` as well),
//! the leading segments become directories and the last one names two files:
//! `<segment>.data` holds the body and `<segment>.json` the content type and
//! metadata. Because `.` is always encoded, those names never clash with a
//! directory made for a longer key, and the temporary files a put writes
//! (whose names start with a dot) never show up in a listing.
//!
//! Meant for development, tests and single-node deployments. Keys that differ
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
        Document, DocumentBody, DocumentKey, DocumentMetadata, ListPage, ListRequest,
        StoredDocument,
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

    /// The directory holding `key`'s files, and the stem of their names.
    fn location(&self, key: &str) -> (PathBuf, String) {
        let mut segments = key.split('/');
        let last = segments.next_back().unwrap_or_default();
        let mut directory = self.root.clone();
        for segment in segments {
            directory.push(encode(segment));
        }
        (directory, encode(last))
    }

    fn data_path(&self, key: &DocumentKey) -> PathBuf {
        let (directory, stem) = self.location(key.as_str());
        directory.join(format!("{stem}{DATA_SUFFIX}"))
    }

    async fn metadata_of(&self, key: &DocumentKey) -> Result<DocumentMetadata, StorageError> {
        let (directory, stem) = self.location(key.as_str());
        let file = fs::metadata(directory.join(format!("{stem}{DATA_SUFFIX}")))
            .await
            .map_err(|error| io_error(key, &error))?;
        let stored = match fs::read(directory.join(format!("{stem}{METADATA_SUFFIX}"))).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| {
                StorageError::Io(format!("metadata of '{key}' is unreadable: {error}"))
            })?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => StoredMetadata::default(),
            Err(error) => return Err(io_error(key, &error)),
        };
        let modified = file.modified().ok();
        let modified_nanos = modified
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();

        Ok(DocumentMetadata {
            key: key.clone(),
            size: file.len(),
            content_type: stored.content_type,
            etag: Some(format!("{:x}-{modified_nanos:x}", file.len())),
            last_modified: modified.map(OffsetDateTime::from),
            metadata: stored.metadata,
        })
    }

    async fn write_files(
        &self,
        key: &DocumentKey,
        document: &Document,
        stored: &[u8],
    ) -> io::Result<()> {
        let (directory, stem) = self.location(key.as_str());
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

    /// Every stored key under the directory the prefix's complete segments
    /// lead to that starts with `prefix`, unsorted.
    async fn keys_with_prefix(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        let (start, key_prefix) = match prefix.rsplit_once('/') {
            Some((directories, _)) => {
                let mut start = self.root.clone();
                for segment in directories.split('/') {
                    start.push(encode(segment));
                }
                (start, format!("{directories}/"))
            }
            None => (self.root.clone(), String::new()),
        };

        let mut keys = Vec::new();
        let mut pending = vec![(start, key_prefix)];
        while let Some((directory, key_prefix)) = pending.pop() {
            let mut entries = match fs::read_dir(&directory).await {
                Ok(entries) => entries,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(StorageError::Io(format!(
                        "cannot list '{}': {error}",
                        directory.display()
                    )));
                }
            };
            while let Some(entry) = entries.next_entry().await.map_err(|error| {
                StorageError::Io(format!("cannot list '{}': {error}", directory.display()))
            })? {
                let name = entry.file_name();
                let Some(name) = name.to_str() else { continue };
                let file_type = entry.file_type().await.map_err(|error| {
                    StorageError::Io(format!("cannot list '{}': {error}", directory.display()))
                })?;
                if file_type.is_dir() {
                    if let Some(segment) = decode(name) {
                        pending.push((entry.path(), format!("{key_prefix}{segment}/")));
                    }
                } else if let Some(segment) = name.strip_suffix(DATA_SUFFIX).and_then(decode) {
                    let key = format!("{key_prefix}{segment}");
                    if key.starts_with(prefix) {
                        keys.push(key);
                    }
                }
            }
        }
        Ok(keys)
    }
}

impl DocumentStorage for FilesystemDocumentStorage {
    fn put<'a>(
        &'a self,
        key: &'a DocumentKey,
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

            // A delete of a neighbouring key may remove the directory between
            // creating it and writing into it; one more try recreates it.
            let mut result = self.write_files(key, &document, &stored).await;
            if matches!(&result, Err(error) if error.kind() == io::ErrorKind::NotFound) {
                result = self.write_files(key, &document, &stored).await;
            }
            result.map_err(|error| io_error(key, &error))?;

            self.metadata_of(key).await
        })
    }

    fn head<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(self.metadata_of(key))
    }

    fn get<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, StoredDocument> {
        Box::pin(async move {
            let metadata = self.metadata_of(key).await?;
            let file = fs::File::open(self.data_path(key))
                .await
                .map_err(|error| io_error(key, &error))?;
            let name = key.to_string();
            let body = stream::try_unfold(file, move |mut file| {
                let name = name.clone();
                async move {
                    let mut chunk = BytesMut::with_capacity(CHUNK_SIZE);
                    let read = file.read_buf(&mut chunk).await.map_err(|error| {
                        StorageError::Io(format!("reading '{name}' failed: {error}"))
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

    fn delete<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let (directory, stem) = self.location(key.as_str());
            for suffix in [DATA_SUFFIX, METADATA_SUFFIX] {
                match fs::remove_file(directory.join(format!("{stem}{suffix}"))).await {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(io_error(key, &error)),
                }
            }
            self.prune(directory).await;
            Ok(())
        })
    }

    fn list(&self, request: ListRequest) -> StorageFuture<'_, ListPage> {
        Box::pin(async move {
            let prefix = request.prefix.unwrap_or_default();
            let mut keys = self.keys_with_prefix(&prefix).await?;
            keys.sort_unstable();
            if let Some(token) = &request.page_token {
                keys.retain(|key| key > token);
            }

            let page_size = request
                .page_size
                .and_then(|size| usize::try_from(size).ok())
                .filter(|size| *size > 0)
                .map_or(MAX_PAGE_SIZE, |size| size.min(MAX_PAGE_SIZE));
            let more = keys.len() > page_size;
            keys.truncate(page_size);

            let mut documents = Vec::with_capacity(keys.len());
            for key in &keys {
                match self
                    .metadata_of(&DocumentKey::from_backend(key.clone()))
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
                next_page_token: if more { keys.last().cloned() } else { None },
            })
        })
    }
}

fn encode(segment: &str) -> String {
    utf8_percent_encode(segment, ENCODE_SET).to_string()
}

/// The key segment a file or directory name stands for, or `None` for a name
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

fn io_error(key: &DocumentKey, error: &io::Error) -> StorageError {
    match error.kind() {
        io::ErrorKind::NotFound => StorageError::NotFound(key.to_string()),
        io::ErrorKind::PermissionDenied => StorageError::Forbidden(format!("'{key}': {error}")),
        _ => StorageError::Io(format!("'{key}': {error}")),
    }
}
