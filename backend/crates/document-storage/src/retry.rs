//! Retry policy shared by backends.

use std::{future::Future, time::Duration};

use crate::error::StorageError;

const MAX_ATTEMPTS: u32 = 4;
const INITIAL_BACKOFF: Duration = Duration::from_millis(200);

/// Runs `operation` until it succeeds or fails with a non-transient error,
/// backing off exponentially for at most four tries. Backends wrap each
/// request in this; `name` only labels the retry log line.
///
/// # Errors
///
/// Returns the last error from `operation` once it is not transient or the
/// tries are used up.
pub async fn with_retry<T, F, Fut>(name: &str, operation: F) -> Result<T, StorageError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, StorageError>>,
{
    let mut attempt = 1;
    let mut backoff = INITIAL_BACKOFF;
    loop {
        match operation().await {
            Err(error) if attempt < MAX_ATTEMPTS && error.is_transient() => {
                tracing::debug!(operation = name, attempt, %error, "retrying document storage call");
                tokio::time::sleep(backoff).await;
                attempt += 1;
                backoff *= 2;
            }
            result => return result,
        }
    }
}
