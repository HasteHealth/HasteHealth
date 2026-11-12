use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Display},
    future::ready,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_util::{StreamExt as _, stream};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use time::OffsetDateTime;

use crate::error::StorageError;

/// Longest name accepted. Google Cloud Storage and S3 allow 1024 bytes for an
/// object name, which a backend builds from the scope and the name.
const MAX_NAME_BYTES: usize = 1024;

/// The tenant and project a document belongs to.
///
/// Every storage call takes one, so a caller reaches only the documents of the
/// project it names: no [`DocumentName`] can spell out another project's
/// document, and a read needs no check of its own. Backends keep the scope
/// apart from the name: columns in Postgres, the leading directories on a
/// filesystem, the leading segments of an object name in a bucket.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentScope {
    tenant: TenantId,
    project: ProjectId,
}

impl DocumentScope {
    /// Both ids match [`haste_jwt::HOSTNAME_ID_PATTERN`] by construction
    /// (lowercase letters, digits and single hyphens), so a backend can use
    /// them as path segments as they are.
    #[must_use]
    pub fn new(tenant: &TenantId, project: &ProjectId) -> Self {
        Self {
            tenant: tenant.clone(),
            project: project.clone(),
        }
    }

    /// Wraps ids read back from a backend. Only backends call this.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::InvalidScope`] when either is not a valid id,
    /// which means something other than this crate wrote to the store.
    pub fn from_backend(tenant: String, project: String) -> Result<Self, StorageError> {
        let invalid = |error: OperationOutcomeError| StorageError::InvalidScope(error.to_string());
        Ok(Self {
            tenant: TenantId::new(tenant).map_err(invalid)?,
            project: ProjectId::new(project).map_err(invalid)?,
        })
    }

    #[must_use]
    pub fn tenant(&self) -> &TenantId {
        &self.tenant
    }

    #[must_use]
    pub fn project(&self) -> &ProjectId {
        &self.project
    }
}

impl Display for DocumentScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.tenant.as_ref(), self.project.as_ref())
    }
}

/// A document's name within its [`DocumentScope`], such as `Binary/123`.
///
/// Names are non-empty UTF-8 of at most 1024 bytes with no control characters,
/// no leading slash and no empty, `.` or `..` segments. Slashes mean nothing
/// to an object store, but they keep related documents together and keep
/// names safe for a filesystem-backed implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentName(String);

impl DocumentName {
    /// Validates `name` against the rules above.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::InvalidName`] naming the broken rule.
    pub fn new(name: impl Into<String>) -> Result<Self, StorageError> {
        let name = name.into();
        let problem = if name.is_empty() {
            Some("must not be empty")
        } else if name.len() > MAX_NAME_BYTES {
            Some("must be at most 1024 bytes")
        } else if name.starts_with('/') {
            Some("must not start with a slash")
        } else if name.chars().any(char::is_control) {
            Some("must not contain control characters")
        } else if name
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
        {
            Some("must not contain empty, '.' or '..' segments")
        } else {
            None
        };

        match problem {
            Some(problem) => Err(StorageError::InvalidName(format!("'{name}' {problem}"))),
            None => Ok(Self(name)),
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

impl Display for DocumentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for DocumentName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for DocumentName {
    type Err = StorageError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::new(name)
    }
}

impl From<DocumentName> for String {
    fn from(name: DocumentName) -> Self {
        name.0
    }
}

/// A document to store: its bytes and what the backend keeps with them.
#[derive(Debug, Clone, Default)]
pub struct Document {
    pub body: Bytes,
    /// MIME type the backend serves the body with; defaults to
    /// `application/octet-stream`.
    pub content_type: Option<String>,
    /// Free-form pairs stored with the object, for example the owning
    /// resource.
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
    pub scope: DocumentScope,
    pub name: DocumentName,
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

    /// A body that is already in memory.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Self::from_stream(stream::once(ready(Ok(bytes.into()))))
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

/// Selects one page of a scope's listing.
#[derive(Debug, Clone, Default)]
pub struct ListRequest {
    /// Token from the previous [`ListPage`]; `None` starts from the beginning.
    pub page_token: Option<String>,
    /// Upper bound on documents per page; the backend applies its own cap.
    pub page_size: Option<u32>,
}

/// One page of a scope's listing, in name order.
#[derive(Debug, Clone)]
pub struct ListPage {
    pub documents: Vec<DocumentMetadata>,
    /// Present when more documents follow.
    pub next_page_token: Option<String>,
}
