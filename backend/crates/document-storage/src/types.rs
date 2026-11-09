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
    /// Checks that both ids can serve as a path segment: non-empty, no slash,
    /// no control character, not `.` or `..`. Haste Health's tenant and
    /// project ids are far stricter than that; the check keeps a backend's
    /// paths safe whatever it is handed.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::InvalidScope`] naming the broken rule.
    pub fn new(tenant: &TenantId, project: &ProjectId) -> Result<Self, StorageError> {
        for (what, id) in [("tenant", tenant.as_ref()), ("project", project.as_ref())] {
            if let Some(problem) = segment_problem(id) {
                return Err(StorageError::InvalidScope(format!(
                    "{what} id '{id}' {problem}"
                )));
            }
        }
        Ok(Self {
            tenant: tenant.clone(),
            project: project.clone(),
        })
    }

    /// Wraps ids read back from a backend without validating them. Only
    /// backends should call this; whatever is in the store is reported as is.
    #[must_use]
    pub fn from_backend(tenant: String, project: String) -> Self {
        Self {
            tenant: TenantId::new(tenant),
            project: ProjectId::new(project),
        }
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

/// Why `segment` cannot be a path segment, if it cannot.
fn segment_problem(segment: &str) -> Option<&'static str> {
    if segment.is_empty() {
        Some("must not be empty")
    } else if segment.contains('/') {
        Some("must not contain a slash")
    } else if segment.chars().any(char::is_control) {
        Some("must not contain control characters")
    } else if matches!(segment, "." | "..") {
        Some("must not be '.' or '..'")
    } else {
        None
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
    /// Only that project's documents. `None` lists every document in the
    /// store, which maintenance such as an orphan sweep needs.
    pub scope: Option<DocumentScope>,
    /// Token from the previous [`ListPage`]; `None` starts from the beginning.
    pub page_token: Option<String>,
    /// Upper bound on documents per page; the backend applies its own cap.
    pub page_size: Option<u32>,
}

/// One page of a listing, ordered by tenant, project and name.
#[derive(Debug, Clone)]
pub struct ListPage {
    pub documents: Vec<DocumentMetadata>,
    /// Present when more documents follow.
    pub next_page_token: Option<String>,
}

/// The page token the filesystem and Postgres backends hand out: the last
/// document's scope and name. Both compare it as a (tenant, project, name)
/// triple, never as one string, since a slash sorts between the characters an
/// id may hold.
#[cfg(any(feature = "filesystem", feature = "postgres"))]
pub(crate) fn page_token(scope: &DocumentScope, name: &str) -> String {
    format!(
        "{}/{}/{name}",
        scope.tenant().as_ref(),
        scope.project().as_ref()
    )
}

/// The triple a [`page_token`] encodes.
///
/// # Errors
///
/// Returns [`StorageError::InvalidPageToken`] for a token of another shape.
#[cfg(any(feature = "filesystem", feature = "postgres"))]
pub(crate) fn parse_page_token(token: &str) -> Result<(String, String, String), StorageError> {
    let mut parts = token.splitn(3, '/');
    if let (Some(tenant), Some(project), Some(name)) = (parts.next(), parts.next(), parts.next())
        && !tenant.is_empty()
        && !project.is_empty()
        && !name.is_empty()
    {
        Ok((tenant.to_string(), project.to_string(), name.to_string()))
    } else {
        Err(StorageError::InvalidPageToken(format!(
            "'{token}' is not a token this backend issued"
        )))
    }
}
