//! Shared CLI plumbing: persisted state (`config`, `secrets`) and the FHIR HTTP `client`
//! built from it. Not user-facing commands themselves — see [`crate::commands`] for those.

pub(crate) mod client;
pub(crate) mod config;
pub(crate) mod secrets;
pub(crate) mod state;
