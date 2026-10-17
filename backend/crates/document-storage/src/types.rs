use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Display},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_util::StreamExt as _;
use time::OffsetDateTime;

use crate::error::StorageError;

/// Longest key accepted. Google Cloud Storage and S3 both allow 1024 bytes.
const MAX_KEY_BYTES: usize = 1024;

/// A validated object key such as `tenant/project/Binary/123`.
///
/// Keys are non-empty UTF-8 of at most 1024 bytes with no control characters,
/// no leading slash and no empty, `.` or `..` segments. Slashes mean nothing
/// to an object store, but they keep related documents listable by prefix and
/// keep keys safe for a filesystem-backed implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentKey(String);

impl DocumentKey {
    /// Validates `key` against the rules above.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::InvalidKey`] naming the broken rule.
    pub fn new(key: impl Into<String>) -> Result<Self, StorageError> {
        let key = key.into();
        let problem = if key.is_empty() {
            Some("must not be empty")
        } else if key.len() > MAX_KEY_BYTES {
            Some("must be at most 1024 bytes")
        } else if key.starts_with('/') {
            Some("must not start with a slash")
        } else if key.chars().any(char::is_control) {
            Some("must not contain control characters")
        } else if key
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
        {
            Some("must not contain empty, '.' or '..' segments")
        } else {
            None
        };

        match problem {
            Some(problem) => Err(StorageError::InvalidKey(format!("'{key}' {problem}"))),
            None => Ok(Self(key)),
        }
    }

    /// Wraps a name read back from a backend without validating it. Only
    /// backends should call this; whatever is in the store is reported as is.
    #[must_use]
    pub fn from_backend(name: String) -> Self {
        Self(name)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DocumentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for DocumentKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for DocumentKey {
    type Err = StorageError;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        Self::new(key)
    }
}

impl From<DocumentKey> for String {
    fn from(key: DocumentKey) -> Self {
        key.0
    }
}

/// A document to store: its bytes and what the backend keeps with them.
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub body: Bytes,
    /// MIME type the backend serves the body with; defaults to
    /// `application/octet-stream`.
    pub content_type: Option<String>,
    /// Free-form pairs stored with the object, for example the owning tenant
    /// and resource.
    pub metadata: BTreeMap<String, String>,
}

impl Document {
    #[must_use]
    pub fn new(body: impl Into<Bytes>) -> Self {
        Self {
            body: body.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// What a backend knows about a stored document without reading its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub key: DocumentKey,
    /// Body length in bytes.
    pub size: u64,
    pub content_type: Option<String>,
    /// Backend version tag; changes whenever the body does.
    pub etag: Option<String>,
    pub last_modified: Option<OffsetDateTime>,
    /// The pairs given as [`Document::metadata`] when the document was stored.
    pub metadata: BTreeMap<String, String>,
}

/// A document body as a stream of chunks. Forward it chunk by chunk (for
/// example into an HTTP response) or gather it with [`DocumentBody::collect`].
pub struct DocumentBody(Pin<Box<dyn Stream<Item = Result<Bytes, StorageError>> + Send>>);

impl DocumentBody {
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, StorageError>> + Send + 'static,
    {
        Self(Box::pin(stream))
    }

    /// Reads the whole body into memory. Check [`DocumentMetadata::size`]
    /// first when the document may be large.
    ///
    /// # Errors
    ///
    /// Returns the first error the stream yields.
    pub async fn collect(mut self) -> Result<Bytes, StorageError> {
        let mut buffer = BytesMut::new();
        while let Some(chunk) = self.0.next().await {
            buffer.extend_from_slice(&chunk?);
        }
        Ok(buffer.freeze())
    }
}

impl Stream for DocumentBody {
    type Item = Result<Bytes, StorageError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(cx)
    }
}

impl Debug for DocumentBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DocumentBody(..)")
    }
}

/// A stored document: its metadata and body.
#[derive(Debug)]
pub struct StoredDocument {
    pub metadata: DocumentMetadata,
    pub body: DocumentBody,
}

/// Selects one page of a listing.
#[derive(Debug, Clone, Default)]
pub struct ListRequest {
    /// Only keys starting with this are returned; `None` lists everything.
    pub prefix: Option<String>,
    /// Token from the previous [`ListPage`]; `None` starts from the beginning.
    pub page_token: Option<String>,
    /// Upper bound on documents per page; the backend applies its own cap.
    pub page_size: Option<u32>,
}

/// One page of a listing, in key order.
#[derive(Debug, Clone)]
pub struct ListPage {
    pub documents: Vec<DocumentMetadata>,
    /// Present when more documents follow.
    pub next_page_token: Option<String>,
}
