//! The JSON API resource shapes this backend reads and writes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The fields of a `storage#object` resource this backend uses.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Object {
    pub name: String,
    /// Byte length, sent as a decimal string.
    pub size: Option<String>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    /// RFC 3339 timestamp of the last change.
    pub updated: Option<String>,
    /// Content version; pinning it on download keeps a read consistent with
    /// the metadata fetched just before.
    pub generation: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

/// One page of `objects.list`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ObjectList {
    #[serde(default)]
    pub items: Vec<Object>,
    pub next_page_token: Option<String>,
}

/// The metadata part of a multipart upload.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UploadMetadata<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<&'a str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: &'a BTreeMap<String, String>,
}
