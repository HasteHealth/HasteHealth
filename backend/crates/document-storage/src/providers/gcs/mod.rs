//! Google Cloud Storage backend, speaking the JSON API over HTTPS.
//!
//! A document is the object `<prefix>/<tenant>/<project>/<name>`: the
//! deployment prefix, if any, then the scope, then the name. A bucket is flat,
//! but that layout lists and shows as a folder per tenant and project.

mod api;
mod auth;
mod http;

use std::{
    fmt::Write as _,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bytes::{Bytes, BytesMut};
use futures_util::TryStreamExt as _;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::{Client, Method, RequestBuilder, Response, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub use auth::GcsCredentials;

use self::{
    api::{Object, ObjectList, UploadMetadata},
    auth::Authenticator,
    http::{check, parse_json, transport},
};
use crate::{
    error::StorageError,
    retry::with_retry,
    traits::{DocumentStorage, StorageFuture},
    types::{
        Document, DocumentBody, DocumentMetadata, DocumentName, DocumentScope, ListPage,
        ListRequest, StoredDocument,
    },
};

const DEFAULT_ENDPOINT: &str = "https://storage.googleapis.com";
const DEFAULT_CONTENT_TYPE: &str = "application/octet-stream";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Longest pause tolerated while waiting for response bytes.
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Everything but RFC 3986 unreserved characters is percent-encoded in object
/// names and query values, so `/` becomes `%2F` as the API requires.
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// Settings for [`GcsDocumentStorage`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GcsConfig {
    pub bucket: String,
    /// Folder-like prefix placed before every object, so one bucket can
    /// serve several deployments.
    #[serde(default)]
    pub prefix: Option<String>,
    /// Application Default Credentials unless set.
    #[serde(default)]
    pub credentials: GcsCredentials,
    /// API root, `https://storage.googleapis.com` unless set. Point it at an
    /// emulator for local development.
    #[serde(default)]
    pub endpoint: Option<String>,
}

/// [`DocumentStorage`] over one Google Cloud Storage bucket.
pub struct GcsDocumentStorage {
    client: Client,
    auth: Authenticator,
    bucket: String,
    /// Prefix without surrounding slashes; empty when unset.
    prefix: String,
    /// `{endpoint}/storage/v1/b/{bucket}/o`
    objects_url: String,
    /// `{endpoint}/upload/storage/v1/b/{bucket}/o?uploadType=multipart`
    upload_url: String,
}

impl GcsDocumentStorage {
    /// Builds the client and obtains a first access token, so rejected
    /// credentials fail here rather than on the first document.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Configuration`] for an unusable configuration or
    /// credentials file, and [`StorageError::Unauthorized`] when the token
    /// endpoint rejects the credentials.
    pub async fn connect(config: GcsConfig) -> Result<Self, StorageError> {
        if config.bucket.is_empty() {
            return Err(StorageError::Configuration(
                "bucket name is empty".to_string(),
            ));
        }
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            .build()
            .map_err(|error| {
                StorageError::Configuration(format!("could not build HTTP client: {error}"))
            })?;
        let auth = Authenticator::new(&config.credentials).await?;
        let endpoint = config
            .endpoint
            .as_deref()
            .unwrap_or(DEFAULT_ENDPOINT)
            .trim_end_matches('/');
        let bucket = config.bucket;

        let storage = Self {
            client,
            auth,
            prefix: config
                .prefix
                .as_deref()
                .map(|prefix| prefix.trim_matches('/'))
                .unwrap_or_default()
                .to_string(),
            objects_url: format!("{endpoint}/storage/v1/b/{bucket}/o"),
            upload_url: format!("{endpoint}/upload/storage/v1/b/{bucket}/o?uploadType=multipart"),
            bucket,
        };
        storage.auth.bearer(&storage.client).await?;
        Ok(storage)
    }

    /// Where the objects of `scope` start, or of every scope when `None`:
    /// the deployment prefix alone.
    fn scope_prefix(&self, scope: Option<&DocumentScope>) -> String {
        let mut prefix = String::new();
        if !self.prefix.is_empty() {
            prefix.push_str(&self.prefix);
            prefix.push('/');
        }
        if let Some(scope) = scope {
            // Writing to a `String` cannot fail.
            let _ = write!(
                prefix,
                "{}/{}/",
                scope.tenant().as_ref(),
                scope.project().as_ref()
            );
        }
        prefix
    }

    /// The object name of a document: the deployment prefix, the scope's
    /// tenant and project, then the name.
    fn object_name(&self, scope: &DocumentScope, name: &DocumentName) -> String {
        let mut object = self.scope_prefix(Some(scope));
        object.push_str(name.as_str());
        object
    }

    /// The scope and name an object name stands for, or `None` for an object
    /// this backend did not write: outside the deployment prefix, or without
    /// a tenant, project and name.
    fn parse_object_name<'n>(&self, object: &'n str) -> Option<(DocumentScope, &'n str)> {
        let rest = if self.prefix.is_empty() {
            object
        } else {
            object.strip_prefix(&self.prefix)?.strip_prefix('/')?
        };
        let mut parts = rest.splitn(3, '/');
        let (tenant, project, name) = (parts.next()?, parts.next()?, parts.next()?);
        if tenant.is_empty() || project.is_empty() || name.is_empty() {
            return None;
        }
        Some((
            DocumentScope::from_backend(tenant.to_string(), project.to_string()),
            name,
        ))
    }

    fn object_url(&self, name: &str) -> String {
        format!("{}/{}", self.objects_url, encode(name))
    }

    async fn request(&self, method: Method, url: &str) -> Result<RequestBuilder, StorageError> {
        let request = self.client.request(method, url);
        Ok(match self.auth.bearer(&self.client).await? {
            Some(token) => request.bearer_auth(token),
            None => request,
        })
    }

    async fn send(&self, request: RequestBuilder, subject: &str) -> Result<Response, StorageError> {
        check(request.send().await.map_err(transport)?, subject).await
    }

    async fn fetch_object(
        &self,
        scope: &DocumentScope,
        name: &DocumentName,
    ) -> Result<Object, StorageError> {
        let url = self.object_url(&self.object_name(scope, name));
        let subject = format!("{scope}/{name}");
        // Shared references are `Copy`, so every retry attempt can move them
        // into its own future. The other operations follow the same pattern.
        let (url, subject) = (&url, subject.as_str());
        with_retry("head", || async move {
            parse_json(
                self.send(self.request(Method::GET, url).await?, subject)
                    .await?,
            )
            .await
        })
        .await
    }

    /// What the API reports about an object, as the metadata of the document
    /// `scope` and `name` address.
    fn metadata_of(scope: DocumentScope, name: DocumentName, object: Object) -> DocumentMetadata {
        DocumentMetadata {
            scope,
            name,
            size: object
                .size
                .as_deref()
                .and_then(|size| size.parse().ok())
                .unwrap_or(0),
            content_type: object.content_type,
            etag: object.etag,
            last_modified: object
                .updated
                .as_deref()
                .and_then(|updated| OffsetDateTime::parse(updated, &Rfc3339).ok()),
            metadata: object.metadata,
        }
    }
}

impl DocumentStorage for GcsDocumentStorage {
    fn put<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
        document: Document,
    ) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(async move {
            let content_type = document
                .content_type
                .as_deref()
                .unwrap_or(DEFAULT_CONTENT_TYPE);
            if !content_type
                .bytes()
                .all(|byte| byte.is_ascii_graphic() || byte == b' ')
            {
                return Err(StorageError::InvalidDocument(
                    "content type must be printable ASCII".to_string(),
                ));
            }

            let object_name = self.object_name(scope, name);
            let metadata = serde_json::to_vec(&UploadMetadata {
                name: &object_name,
                content_type: document.content_type.as_deref(),
                metadata: &document.metadata,
            })
            .map_err(|error| {
                StorageError::InvalidDocument(format!("metadata is not serializable: {error}"))
            })?;
            let boundary = boundary();
            let body = multipart_related(&boundary, &metadata, content_type, &document.body);

            let subject = format!("{scope}/{name}");
            let (body, boundary, subject) = (&body, &boundary, subject.as_str());
            let object: Object = with_retry("put", || async move {
                let request = self
                    .request(Method::POST, &self.upload_url)
                    .await?
                    .header(
                        CONTENT_TYPE,
                        format!("multipart/related; boundary={boundary}"),
                    )
                    .body(body.clone());
                parse_json(self.send(request, subject).await?).await
            })
            .await?;
            Ok(Self::metadata_of(scope.clone(), name.clone(), object))
        })
    }

    fn head<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(async move {
            let object = self.fetch_object(scope, name).await?;
            Ok(Self::metadata_of(scope.clone(), name.clone(), object))
        })
    }

    fn get<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, StoredDocument> {
        Box::pin(async move {
            let object = self.fetch_object(scope, name).await?;
            let url = match &object.generation {
                Some(generation) => format!(
                    "{}?alt=media&generation={generation}",
                    self.object_url(&object.name)
                ),
                None => format!("{}?alt=media", self.object_url(&object.name)),
            };

            let subject = format!("{scope}/{name}");
            let (url, subject) = (&url, subject.as_str());
            let response = with_retry("get", || async move {
                self.send(self.request(Method::GET, url).await?, subject)
                    .await
            })
            .await?;

            Ok(StoredDocument {
                metadata: Self::metadata_of(scope.clone(), name.clone(), object),
                body: DocumentBody::from_stream(response.bytes_stream().map_err(transport)),
            })
        })
    }

    fn delete<'a>(
        &'a self,
        scope: &'a DocumentScope,
        name: &'a DocumentName,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let url = self.object_url(&self.object_name(scope, name));
            let subject = format!("{scope}/{name}");
            let (url, subject) = (&url, subject.as_str());
            let result = with_retry("delete", || async move {
                self.send(self.request(Method::DELETE, url).await?, subject)
                    .await
            })
            .await;
            match result {
                Ok(_) | Err(StorageError::NotFound(_)) => Ok(()),
                Err(error) => Err(error),
            }
        })
    }

    fn list(&self, request: ListRequest) -> StorageFuture<'_, ListPage> {
        Box::pin(async move {
            let prefix = self.scope_prefix(request.scope.as_ref());
            let mut query = vec![format!("prefix={}", encode(&prefix))];
            if let Some(token) = &request.page_token {
                query.push(format!("pageToken={}", encode(token)));
            }
            if let Some(size) = request.page_size {
                query.push(format!("maxResults={size}"));
            }
            let url = format!("{}?{}", self.objects_url, query.join("&"));

            let url = &url;
            let page: ObjectList = with_retry("list", || async move {
                parse_json(
                    self.send(self.request(Method::GET, url).await?, &self.bucket)
                        .await?,
                )
                .await
            })
            .await?;

            // Objects outside the layout were not written by this backend;
            // they are left out, not reported.
            let documents = page
                .items
                .into_iter()
                .filter_map(|object| {
                    let Some((scope, name)) = self.parse_object_name(&object.name) else {
                        tracing::debug!(object = %object.name, "object outside the document layout, skipped");
                        return None;
                    };
                    let name = DocumentName::from_backend(name.to_string());
                    Some(Self::metadata_of(scope, name, object))
                })
                .collect();
            Ok(ListPage {
                documents,
                next_page_token: page.next_page_token,
            })
        })
    }
}

fn encode(value: &str) -> String {
    utf8_percent_encode(value, ENCODE_SET).to_string()
}

/// A `multipart/related` upload body: the JSON metadata part, then the bytes.
fn multipart_related(boundary: &str, metadata: &[u8], content_type: &str, body: &[u8]) -> Bytes {
    let mut out = BytesMut::with_capacity(metadata.len() + body.len() + 256);
    out.extend_from_slice(
        format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n").as_bytes(),
    );
    out.extend_from_slice(metadata);
    out.extend_from_slice(
        format!("\r\n--{boundary}\r\nContent-Type: {content_type}\r\n\r\n").as_bytes(),
    );
    out.extend_from_slice(body);
    out.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    out.freeze()
}

/// A part boundary whoever produced the document could not have predicted,
/// so it cannot occur inside the body.
fn boundary() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("haste-document-{}-{nanos:x}", std::process::id())
}
