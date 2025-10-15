use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oxidized_config::{Config, ConfigType, get_config};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use rand::rngs::OsRng;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
    traits::PublicKeyParts,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{path::Path, sync::LazyLock};

static PRIVATE_KEY_FILENAME: &str = "private_key.pem";
static PUBLIC_KEY_FILENAME: &str = "public_key.pem";

pub fn create_certifications(config: &dyn Config) -> Result<(), OperationOutcomeError> {
    let certificate_dir = config.get("CERTIFICATION_DIR").unwrap();
    let dir: &Path = Path::new(&certificate_dir);

    let mut rng = OsRng;
    let bits = 2048;

    let private_key_file = dir.join(PRIVATE_KEY_FILENAME);
    let public_key_file = dir.join(PUBLIC_KEY_FILENAME);

    // If no private key than write.
    if !private_key_file.exists() {
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        std::fs::create_dir_all(certificate_dir).unwrap();
        std::fs::write(
            private_key_file,
            priv_key.to_pkcs1_pem(LineEnding::default()).unwrap(),
        )
        .map_err(|e| OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string()))?;

        std::fs::write(
            public_key_file,
            pub_key.to_pkcs1_pem(LineEnding::default()).unwrap(),
        )
        .map_err(|e| OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string()))?;
    }

    Ok(())
}

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
    kid: String,

    alg: JSONWebKeyAlgorithm,
    kty: JSONWebKeyType,
    // Base64 URL SAFE
    e: String,
    n: String,
    x5t: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JSONWebKeySet {
    keys: Vec<JSONWebKey>,
}

pub static JWK_SET: LazyLock<JSONWebKeySet> = LazyLock::new(|| {
    let config = get_config(ConfigType::Environment);
    let certificate_dir = config.get("CERTIFICATION_DIR").unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);
    let rsa_private = RsaPrivateKey::from_pkcs1_pem(
        &std::fs::read_to_string(&cert_dir.join(PRIVATE_KEY_FILENAME)).unwrap(),
    )
    .unwrap();
    let rsa_public_key = rsa_private.to_public_key();

    let mut hasher = Sha1::new();
    hasher.update(rsa_public_key.to_pkcs1_der().unwrap().as_bytes());
    let x5t = hasher.finalize();

    let rsa_public = JSONWebKey {
        kid: URL_SAFE_NO_PAD.encode(&x5t),
        alg: JSONWebKeyAlgorithm::RS256,
        kty: JSONWebKeyType::RSA,
        e: URL_SAFE_NO_PAD.encode(&rsa_public_key.e().clone().to_bytes_be()),
        n: URL_SAFE_NO_PAD.encode(&rsa_public_key.n().clone().to_bytes_be()),
        x5t: Some(URL_SAFE_NO_PAD.encode(&x5t)),
    };

    JSONWebKeySet {
        keys: vec![rsa_public],
    }
});

#[allow(unused)]
// Only used if an environment.
static DECODING_KEY: LazyLock<jsonwebtoken::DecodingKey> = LazyLock::new(|| {
    // let key = CERTIFICATES.public_key.clone();
    let config = get_config(ConfigType::Environment);
    let certificate_dir = config.get("CERTIFICATION_DIR").unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);
    jsonwebtoken::DecodingKey::from_rsa_pem(
        &std::fs::read(cert_dir.join(PUBLIC_KEY_FILENAME)).unwrap(),
    )
    .unwrap()
});

static ENCODING_KEY: LazyLock<jsonwebtoken::EncodingKey> = LazyLock::new(|| {
    // let key = CERTIFICATES.public_key.clone();
    let config = get_config(ConfigType::Environment);
    let certificate_dir = config.get("CERTIFICATION_DIR").unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);
    jsonwebtoken::EncodingKey::from_rsa_pem(
        &std::fs::read(cert_dir.join(PRIVATE_KEY_FILENAME)).unwrap(),
    )
    .unwrap()
});

#[allow(unused)]
pub fn decoding_key() -> &'static jsonwebtoken::DecodingKey {
    &*DECODING_KEY
}

pub fn encoding_key() -> &'static jsonwebtoken::EncodingKey {
    &*ENCODING_KEY
}
