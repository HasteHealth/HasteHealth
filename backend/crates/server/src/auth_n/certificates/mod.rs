use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

use crate::config::ServerConfig;

pub mod providers;
pub mod traits;

#[derive(Serialize, Deserialize, Debug)]
pub enum JSONWebKeyAlgorithm {
    RS256,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JSONWebKeyType {
    RSA,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONWebKey {
    pub kid: String,

    pub alg: JSONWebKeyAlgorithm,
    pub kty: JSONWebKeyType,
    // Base64 URL SAFE
    pub e: String,
    pub n: String,
    pub x5t: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONWebKeySet {
    pub keys: Vec<JSONWebKey>,
}

static CERTIFICATION_PROVIDER: OnceLock<Arc<dyn traits::CertificationProvider>> = OnceLock::new();

pub fn get_certification_provider(config: &ServerConfig) -> Arc<dyn traits::CertificationProvider> {
    CERTIFICATION_PROVIDER
        .get_or_init(|| {
            Arc::new(
                providers::local::LocalCertifications::new(config)
                    .expect("Failed to create LocalCertifications"),
            ) as Arc<dyn traits::CertificationProvider>
        })
        .clone()
}
