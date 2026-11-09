//! Removal of Binary documents whose resource version was never written.

use std::{collections::HashSet, time::Duration};

use haste_document_storage::ListRequest;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::pg::PGConnection;
use serde::Serialize;
use sqlx::types::time::OffsetDateTime;

use super::{BinaryStorage, MARKER_URL};

/// Documents listed per storage request.
const PAGE_SIZE: u32 = 1000;

/// What a sweep found and did.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SweepReport {
    /// Documents looked at.
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
}

/// Removes documents in `storage` that no resource version in `repo` refers
/// to, once they are older than `grace`. It walks every scope the store holds
/// and checks each page of a scope's documents against that tenant and
/// project's rows with one query that the repository's partial index on Binary
/// rows serves, so the sweep costs a probe per document. Such a document comes
/// from a write whose row never committed (a rolled back transaction, a write
/// that failed after the upload, or a crash between the two) or from a project
/// that was deleted while its documents lived outside the repository database.
/// `grace` must comfortably exceed the longest transaction, since a document
/// is uploaded before its row commits.
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
    for scope in storage.storage().scopes().await? {
        let mut page_token = None;
        loop {
            let page = storage
                .storage()
                .list(
                    &scope,
                    ListRequest {
                        page_token,
                        page_size: Some(PAGE_SIZE),
                    },
                )
                .await?;

            let names = page
                .documents
                .iter()
                .map(|document| document.name.as_str())
                .collect::<Vec<_>>();
            let in_use: HashSet<String> = repo
                .binary_data_extension_values(scope.tenant(), scope.project(), MARKER_URL, &names)
                .await?;

            for document in &page.documents {
                report.examined += 1;
                if in_use.contains(document.name.as_str()) {
                    report.in_use += 1;
                } else if document
                    .last_modified
                    .is_none_or(|modified| modified > cutoff)
                {
                    report.recent += 1;
                } else {
                    report.orphaned += 1;
                    if !dry_run {
                        storage.storage().delete(&scope, &document.name).await?;
                        report.removed += 1;
                    }
                }
            }

            match page.next_page_token {
                Some(token) => page_token = Some(token),
                None => break,
            }
        }
    }
    Ok(report)
}
