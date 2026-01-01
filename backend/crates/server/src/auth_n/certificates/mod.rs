use haste_config::{ConfigType, get_config};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};

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

static CERTIFICATION_PROVIDER: LazyLock<Arc<dyn traits::CertificationProvider>> =
    LazyLock::new(|| {
        let config = get_config(ConfigType::Environment);
        Arc::new(
            providers::local::LocalCertifications::new(config.as_ref())
                .expect("Failed to create LocalCertifications"),
        ) as Arc<dyn traits::CertificationProvider>
    });

pub fn get_certification_provider() -> Arc<dyn traits::CertificationProvider> {
    CERTIFICATION_PROVIDER.clone()
}
