//! One-time email codes for sign-up and login, kept in the visitor's session.
//! Nothing touches the tenant or user tables until the address is verified,
//! and a code only works in the browser that asked for it.

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_rate_limit::{RateLimit, RateLimitError};
use haste_repository::utilities::generate_id;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tower_sessions::Session;

pub const CODE_DIGITS: usize = 6;
pub const CODE_VALID_FOR: Duration = Duration::from_secs(10 * 60);
pub const MAX_ATTEMPTS: u8 = 5;
/// How long a verified address may still pick or create a tenant.
const VERIFIED_VALID_FOR: Duration = Duration::from_secs(10 * 60);

const PENDING_KEY: &str = "email_code_pending";
const VERIFIED_KEY: &str = "email_code_verified";

/// Which form asked for the code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Purpose {
    Signup,
    Login,
}

/// The code a session is waiting on. Only a salted hash of it is stored. The
/// count of wrong guesses is not kept here: a session is a per-request copy, so
/// a counter in it is lost under concurrent guesses. [`check`] counts guesses
/// in the shared, serialized rate limiter instead.
#[derive(Serialize, Deserialize)]
pub struct PendingCode {
    pub email: String,
    pub purpose: Purpose,
    salt: String,
    code_hash: String,
    expires_at: u64,
}

/// An address whose code was entered correctly.
#[derive(Clone, Serialize, Deserialize)]
pub struct VerifiedEmail {
    pub email: String,
    pub purpose: Purpose,
    expires_at: u64,
}

pub enum CheckOutcome {
    Verified(VerifiedEmail),
    Incorrect {
        attempts_left: u8,
    },
    Expired,
    TooManyAttempts,
    /// No code was requested in this session.
    NotPending,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn session_error(what: &str) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::exception(), format!("Session error: {what}."))
}

fn hash_code(salt: &str, code: &str) -> String {
    format!("{:x}", Sha256::digest(format!("{salt}:{code}")))
}

fn generate_code() -> String {
    let max = 10u32.pow(CODE_DIGITS as u32);
    format!(
        "{:0width$}",
        rand::thread_rng().gen_range(0..max),
        width = CODE_DIGITS
    )
}

async fn store_pending(
    session: &Session,
    pending: &PendingCode,
) -> Result<(), OperationOutcomeError> {
    session
        .insert(PENDING_KEY, pending)
        .await
        .map_err(|_| session_error("failed to store the pending code"))
}

/// Starts (or restarts) verification for `email` and returns the code to send.
pub async fn start(
    session: &Session,
    email: &str,
    purpose: Purpose,
) -> Result<String, OperationOutcomeError> {
    let code = generate_code();
    let salt = generate_id(Some(16));

    store_pending(
        session,
        &PendingCode {
            email: email.to_string(),
            purpose,
            code_hash: hash_code(&salt, &code),
            salt,
            expires_at: now() + CODE_VALID_FOR.as_secs(),
        },
    )
    .await?;

    Ok(code)
}

/// The code this session is waiting on, expired or not, so the page can still
/// offer a resend.
pub async fn pending(session: &Session) -> Result<Option<PendingCode>, OperationOutcomeError> {
    session
        .get::<PendingCode>(PENDING_KEY)
        .await
        .map_err(|_| session_error("failed to read the pending code"))
}

/// A correct code consumes the pending entry and marks the address verified.
///
/// Every guess spends one point of a shared [`MAX_ATTEMPTS`] budget before the
/// code is compared, so a code can be compared at most [`MAX_ATTEMPTS`] times
/// however many requests run at once. The budget is counted in `rate_limit`
/// with [`RateLimit::check_uncached`], which serializes concurrent spends in
/// the database; a counter kept in the session would be lost under a burst,
/// because each request holds its own copy. The key is the code's salted hash,
/// so a fresh code (a resend) starts over, and the key is never guessable or
/// shared between addresses.
pub async fn check(
    session: &Session,
    rate_limit: &dyn RateLimit,
    submitted: &str,
) -> Result<CheckOutcome, OperationOutcomeError> {
    let Some(pending) = pending(session).await? else {
        return Ok(CheckOutcome::NotPending);
    };

    if now() >= pending.expires_at {
        return Ok(CheckOutcome::Expired);
    }

    let attempt_key = format!("email_code_attempt:{}", pending.code_hash);
    let spent = match rate_limit
        .check_uncached(
            &attempt_key,
            MAX_ATTEMPTS as i32,
            1,
            CODE_VALID_FOR.as_secs() as i32,
        )
        .await
    {
        Ok(spent) => spent,
        Err(RateLimitError::Exceeded) => return Ok(CheckOutcome::TooManyAttempts),
        Err(RateLimitError::Error(error)) => {
            tracing::error!(error, "email code attempt limit check failed");
            return Err(session_error("failed to check the attempt limit"));
        }
    };

    if hash_code(&pending.salt, submitted.trim()) != pending.code_hash {
        return Ok(match MAX_ATTEMPTS as i32 - spent {
            left if left <= 0 => CheckOutcome::TooManyAttempts,
            left => CheckOutcome::Incorrect {
                attempts_left: left as u8,
            },
        });
    }

    session
        .remove::<PendingCode>(PENDING_KEY)
        .await
        .map_err(|_| session_error("failed to clear the pending code"))?;

    let verified = VerifiedEmail {
        email: pending.email,
        purpose: pending.purpose,
        expires_at: now() + VERIFIED_VALID_FOR.as_secs(),
    };

    session
        .insert(VERIFIED_KEY, &verified)
        .await
        .map_err(|_| session_error("failed to store the verified address"))?;

    Ok(CheckOutcome::Verified(verified))
}

/// The address verified in this session, while still fresh.
pub async fn verified(session: &Session) -> Result<Option<VerifiedEmail>, OperationOutcomeError> {
    let verified = session
        .get::<VerifiedEmail>(VERIFIED_KEY)
        .await
        .map_err(|_| session_error("failed to read the verified address"))?;

    Ok(verified.filter(|verified| now() < verified.expires_at))
}

/// Forgets the verified address once it has been used.
pub async fn clear_verified(session: &Session) -> Result<(), OperationOutcomeError> {
    session
        .remove::<VerifiedEmail>(VERIFIED_KEY)
        .await
        .map(|_| ())
        .map_err(|_| session_error("failed to clear the verified address"))
}
