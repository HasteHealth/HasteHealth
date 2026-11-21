//! Replay protection for JWT client assertions (RFC 7523 section 3, item 7):
//! a `jti` is accepted once per client and remembered until the assertion
//! that carried it has expired.

use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use std::time::Duration;

pub trait ClientAssertionReplayProvider {
    /// Records that `client_id` authenticated with the assertion `jti`.
    ///
    /// `Ok(true)` when the `jti` is new. `Ok(false)` when an assertion with
    /// this `jti` was already accepted and could still be replayed: the
    /// caller refuses it. The record is kept for `lifetime`, how long the
    /// assertion stays valid, and is discarded afterwards.
    fn record_client_assertion_jti(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        client_id: &str,
        jti: &str,
        lifetime: Duration,
    ) -> impl std::future::Future<Output = Result<bool, OperationOutcomeError>> + Send;
}
