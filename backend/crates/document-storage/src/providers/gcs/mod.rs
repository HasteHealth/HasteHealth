//! Google Cloud Storage backend, speaking the JSON API over HTTPS.

mod api;
mod auth;
mod http;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bytes::{Bytes, BytesMut};
use futures_util::TryStreamExt as _;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::{Client, Method, RequestBuilder, Response, header::CONTENT_TYPE};
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
        Document, DocumentBody, DocumentKey, DocumentMetadata, ListPage, ListRequest,
        StoredDocument,
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
#[derive(Debug, Clone)]
pub struct GcsConfig {
    pub bucket: String,
    /// Folder-like prefix placed before every key, so one bucket can serve
    /// several deployments.
    pub prefix: Option<String>,
    pub credentials: GcsCredentials,
    /// API root, `https://storage.googleapis.com` unless set. Point it at an
    /// emulator for local development.
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

    /// The object name for a key, with the deployment prefix applied.
    fn object_name(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}/{key}", self.prefix)
        }
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

    async fn fetch_object(&self, key: &DocumentKey) -> Result<Object, StorageError> {
        let url = self.object_url(&self.object_name(key.as_str()));
        // Shared references are `Copy`, so every retry attempt can move them
        // into its own future. The other operations follow the same pattern.
        let (url, key) = (&url, key.as_str());
        with_retry("head", || async move {
            parse_json(
                self.send(self.request(Method::GET, url).await?, key)
                    .await?,
            )
            .await
        })
        .await
    }

    fn metadata_of(&self, object: Object) -> DocumentMetadata {
        let key = if self.prefix.is_empty() {
            object.name.as_str()
        } else {
            object
                .name
                .strip_prefix(&self.prefix)
                .and_then(|rest| rest.strip_prefix('/'))
                .unwrap_or(&object.name)
        };
        DocumentMetadata {
            key: DocumentKey::from_backend(key.to_string()),
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
        key: &'a DocumentKey,
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

            let name = self.object_name(key.as_str());
            let metadata = serde_json::to_vec(&UploadMetadata {
                name: &name,
                content_type: document.content_type.as_deref(),
                metadata: &document.metadata,
            })
            .map_err(|error| {
                StorageError::InvalidDocument(format!("metadata is not serializable: {error}"))
            })?;
            let boundary = boundary();
            let body = multipart_related(&boundary, &metadata, content_type, &document.body);

            let (body, boundary, key) = (&body, &boundary, key.as_str());
            let object: Object = with_retry("put", || async move {
                let request = self
                    .request(Method::POST, &self.upload_url)
                    .await?
                    .header(
                        CONTENT_TYPE,
                        format!("multipart/related; boundary={boundary}"),
                    )
                    .body(body.clone());
                parse_json(self.send(request, key).await?).await
            })
            .await?;
            Ok(self.metadata_of(object))
        })
    }

    fn head<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, DocumentMetadata> {
        Box::pin(async move { Ok(self.metadata_of(self.fetch_object(key).await?)) })
    }

    fn get<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, StoredDocument> {
        Box::pin(async move {
            let object = self.fetch_object(key).await?;
            let url = match &object.generation {
                Some(generation) => format!(
                    "{}?alt=media&generation={generation}",
                    self.object_url(&object.name)
                ),
                None => format!("{}?alt=media", self.object_url(&object.name)),
            };

            let (url, key) = (&url, key.as_str());
            let response = with_retry("get", || async move {
                self.send(self.request(Method::GET, url).await?, key).await
            })
            .await?;

            Ok(StoredDocument {
                metadata: self.metadata_of(object),
                body: DocumentBody::from_stream(response.bytes_stream().map_err(transport)),
            })
        })
    }

    fn delete<'a>(&'a self, key: &'a DocumentKey) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let url = self.object_url(&self.object_name(key.as_str()));
            let (url, key) = (&url, key.as_str());
            let result = with_retry("delete", || async move {
                self.send(self.request(Method::DELETE, url).await?, key)
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
            let prefix = self.object_name(request.prefix.as_deref().unwrap_or_default());
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

            Ok(ListPage {
                documents: page
                    .items
                    .into_iter()
                    .map(|object| self.metadata_of(object))
                    .collect(),
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
