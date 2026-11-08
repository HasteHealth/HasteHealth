//! Removal of Binary documents whose resource version was never written.

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use haste_document_storage::{DocumentMetadata, ListRequest};
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::pg::PGConnection;
use serde::Serialize;
use sqlx::types::time::OffsetDateTime;

use super::{BinaryStorage, MARKER_URL, parse_document_key};

/// Documents listed per storage request.
const PAGE_SIZE: u32 = 1000;

/// What a sweep found and did.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SweepReport {
    /// Binary documents looked at.
    pub examined: u64,
    /// Documents with a resource version: kept.
    pub in_use: u64,
    /// Documents without one but younger than the grace period, which may
    /// belong to a write still in progress: kept.
    pub recent: u64,
    /// Documents without one and older than the grace period.
    pub orphaned: u64,
    /// Orphans removed. Zero on a dry run.
    pub removed: u64,
    /// Keys that do not name a Binary document: left alone.
    pub skipped: u64,
}

/// Removes Binary documents in `storage` that no resource version in `repo`
/// refers to, once they are older than `grace`. Such a document comes from a
/// write whose row never committed: a rolled back transaction, a write that
/// failed after the upload, or a crash between the two. `grace` must
/// comfortably exceed the longest transaction, since a document is uploaded
/// before its row commits.
///
/// With `dry_run` nothing is removed and the report counts what would be.
///
/// # Errors
///
/// Fails when storage or the database fails.
pub async fn sweep(
    repo: &PGConnection,
    storage: &BinaryStorage,
    grace: Duration,
    dry_run: bool,
) -> Result<SweepReport, OperationOutcomeError> {
    let cutoff = OffsetDateTime::now_utc() - grace;

    let mut report = SweepReport::default();
    let mut page_token = None;
    loop {
        let page = storage
            .storage()
            .list(ListRequest {
                prefix: None,
                page_token,
                page_size: Some(PAGE_SIZE),
            })
            .await?;

        let mut by_project: HashMap<(&str, &str), Vec<&DocumentMetadata>> = HashMap::new();
        for document in &page.documents {
            report.examined += 1;
            match parse_document_key(document.key.as_str()) {
                Some(scope) => by_project.entry(scope).or_default().push(document),
                None => report.skipped += 1,
            }
        }

        for ((tenant, project), documents) in by_project {
            let keys = documents
                .iter()
                .map(|document| document.key.as_str())
                .collect::<Vec<_>>();
            let in_use: HashSet<String> = repo
                .primitive_extension_values(
                    &TenantId::new(tenant.to_string()),
                    &ProjectId::new(project.to_string()),
                    &ResourceType::Binary,
                    "data",
                    MARKER_URL,
                    &keys,
                )
                .await?;

            for document in documents {
                if in_use.contains(document.key.as_str()) {
                    report.in_use += 1;
                } else if document
                    .last_modified
                    .is_none_or(|modified| modified > cutoff)
                {
                    report.recent += 1;
                } else {
                    report.orphaned += 1;
                    if !dry_run {
                        storage.storage().delete(&document.key).await?;
                        report.removed += 1;
                    }
                }
            }
        }

        match page.next_page_token {
            Some(token) => page_token = Some(token),
            None => return Ok(report),
        }
    }
}
