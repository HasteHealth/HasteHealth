//! Removal of Binary documents whose resource version was never written.

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use haste_document_storage::{DocumentMetadata, ListRequest};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::Serialize;
use sqlx::types::time::OffsetDateTime;

use crate::{
    binary_storage::parse_document_key,
    pg::{PGConnection, StoreError},
};

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
    /// Keys that do not name a Binary version: left alone.
    pub skipped: u64,
}

impl PGConnection {
    /// Removes Binary documents in document storage that no resource version
    /// refers to, once they are older than `grace`. Such a document comes
    /// from a write whose row never committed: a rolled back transaction, a
    /// failed insert, or a crash between the upload and the commit. `grace`
    /// must comfortably exceed the longest transaction, since a document is
    /// uploaded before its row commits.
    ///
    /// With `dry_run` nothing is removed and the report counts what would be.
    ///
    /// # Errors
    ///
    /// Fails when document storage is not configured, when called inside a
    /// transaction, or when storage or the database fails.
    pub async fn sweep_binary_documents(
        &self,
        grace: Duration,
        dry_run: bool,
    ) -> Result<SweepReport, OperationOutcomeError> {
        let Some(storage) = self.binary_storage() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Document storage is not configured.".to_string(),
            ));
        };
        let PGConnection::Pool(pool, _) = self else {
            return Err(StoreError::TransactionError.into());
        };
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

            let mut by_project: HashMap<(&str, &str), Vec<(&DocumentMetadata, &str)>> =
                HashMap::new();
            for document in &page.documents {
                report.examined += 1;
                match parse_document_key(document.key.as_str()) {
                    Some((tenant, project, version_id)) => by_project
                        .entry((tenant, project))
                        .or_default()
                        .push((document, version_id)),
                    None => report.skipped += 1,
                }
            }

            for ((tenant, project), documents) in by_project {
                let version_ids = documents
                    .iter()
                    .map(|(_, version_id)| *version_id)
                    .collect::<Vec<_>>();
                let written = sqlx::query_scalar::<_, String>(
                    r"
                        SELECT version_id
                        FROM resources
                        WHERE tenant = $1 AND project = $2 AND version_id = ANY($3::text[])
                    ",
                )
                .bind(tenant)
                .bind(project)
                .bind(&version_ids)
                .fetch_all(pool)
                .await
                .map_err(StoreError::from)?
                .into_iter()
                .collect::<HashSet<_>>();

                for (document, version_id) in documents {
                    if written.contains(version_id) {
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
}
