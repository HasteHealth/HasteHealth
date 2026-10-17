//! Storage backends, each behind a cargo feature.

#[cfg(feature = "gcs")]
pub mod gcs;
