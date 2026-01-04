use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_config::Config;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use rand::rngs::OsRng;
use rsa::{
    RsaPrivateKey,
    pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
    traits::PublicKeyParts,
};
use sha1::{Digest, Sha1};
use std::{path::Path, sync::Arc};

use crate::{
    ServerEnvironmentVariables,
    auth_n::certificates::{
        JSONWebKey, JSONWebKeyAlgorithm, JSONWebKeySet, JSONWebKeyType,
        traits::CertificationProvider,
    },
};

static PRIVATE_KEY_FILENAME: &str = "private_key.pem";

fn create_jwk_set(
    config: &dyn Config<ServerEnvironmentVariables>,
) -> Result<JSONWebKeySet, OperationOutcomeError> {
    let certificate_dir = config
        .get(ServerEnvironmentVariables::CertificationDir)
        .unwrap();
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

    Ok(JSONWebKeySet {
        keys: vec![rsa_public],
    })
}

fn create_decoding_key(
    config: &dyn Config<ServerEnvironmentVariables>,
) -> Result<jsonwebtoken::DecodingKey, OperationOutcomeError> {
    // let key = CERTIFICATES.public_key.clone();
    let certificate_dir = config
        .get(ServerEnvironmentVariables::CertificationDir)
        .unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);

    let rsa_private = RsaPrivateKey::from_pkcs1_pem(
        &std::fs::read_to_string(&cert_dir.join(PRIVATE_KEY_FILENAME)).unwrap(),
    )
    .unwrap();

    let rsa_public_key = rsa_private.to_public_key();

    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(
        rsa_public_key
            .to_pkcs1_pem(LineEnding::default())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    Ok(decoding_key)
}

fn create_encoding_key(
    config: &dyn Config<ServerEnvironmentVariables>,
) -> Result<jsonwebtoken::EncodingKey, OperationOutcomeError> {
    let certificate_dir = config
        .get(ServerEnvironmentVariables::CertificationDir)
        .unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);
    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(
        &std::fs::read(cert_dir.join(PRIVATE_KEY_FILENAME)).unwrap(),
    )
    .unwrap();

    Ok(encoding_key)
}

fn create_certifications_if_needed(
    config: &dyn Config<ServerEnvironmentVariables>,
) -> Result<(), OperationOutcomeError> {
    let certificate_dir = config.get(ServerEnvironmentVariables::CertificationDir)?;

    let dir: &Path = Path::new(&certificate_dir);

    let mut rng = OsRng;
    let bits: usize = 2048;

    let private_key_file = dir.join(PRIVATE_KEY_FILENAME);

    // If no private key than write.
    if !private_key_file.exists() {
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        std::fs::create_dir_all(certificate_dir).unwrap();
        std::fs::write(
            private_key_file,
            priv_key.to_pkcs1_pem(LineEnding::default()).unwrap(),
        )
        .map_err(|e| OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string()))?;
    }

    Ok(())
}

pub struct LocalCertifications {
    decoding_key: Arc<jsonwebtoken::DecodingKey>,
    encoding_key: Arc<jsonwebtoken::EncodingKey>,
    jwk_set: Arc<JSONWebKeySet>,
}

impl LocalCertifications {
    pub fn new(
        config: &dyn Config<ServerEnvironmentVariables>,
    ) -> Result<Self, OperationOutcomeError> {
        create_certifications_if_needed(config)?;
        Ok(LocalCertifications {
            decoding_key: Arc::new(create_decoding_key(config)?),
            encoding_key: Arc::new(create_encoding_key(config)?),
            jwk_set: Arc::new(create_jwk_set(config)?),
        })
    }
}

impl CertificationProvider for LocalCertifications {
    fn decoding_key(&self) -> Arc<jsonwebtoken::DecodingKey> {
        self.decoding_key.clone()
    }

    fn encoding_key(&self) -> Arc<jsonwebtoken::EncodingKey> {
        self.encoding_key.clone()
    }

    fn jwk_set(&self) -> Arc<JSONWebKeySet> {
        self.jwk_set.clone()
    }
}
