//! Storage backends, each behind a cargo feature.

#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(feature = "gcs")]
pub mod gcs;
#[cfg(feature = "postgres")]
pub mod postgres;
