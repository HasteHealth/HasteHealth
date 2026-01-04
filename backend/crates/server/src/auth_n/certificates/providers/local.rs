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
use walkdir::{DirEntry, WalkDir};

use crate::{
    ServerEnvironmentVariables,
    auth_n::certificates::{
        JSONWebKey, JSONWebKeyAlgorithm, JSONWebKeySet, JSONWebKeyType,
        traits::{CertificationProvider, DecodingKey, EncodingKey},
    },
};

fn derive_kid(cert_path: &Path) -> String {
    let file_name = Path::file_stem(cert_path)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let chunks = file_name.split("_").collect::<Vec<&str>>();
    chunks.get(0).unwrap().to_string()
}

fn get_sorted_private_cert_paths(config: &dyn Config<ServerEnvironmentVariables>) -> Vec<DirEntry> {
    let certificate_dir = config
        .get(ServerEnvironmentVariables::CertificationDir)
        .unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);
    let walker = WalkDir::new(cert_dir).into_iter();
    let mut entries = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().unwrap().is_file())
        .filter(|e| e.file_name().to_str().unwrap().ends_with(".pem"))
        .collect::<Vec<DirEntry>>();

    entries.sort_by(|a, b| {
        let a_chunks = Path::file_stem(a.path())
            .unwrap()
            .to_str()
            .unwrap()
            .split("_")
            .collect::<Vec<&str>>();
        let b_chunks = Path::file_stem(b.path())
            .unwrap()
            .to_str()
            .unwrap()
            .split("_")
            .collect::<Vec<&str>>();

        let date_a = chrono::DateTime::parse_from_rfc3339(a_chunks.get(1).unwrap()).unwrap();
        let date_b = chrono::DateTime::parse_from_rfc3339(b_chunks.get(1).unwrap()).unwrap();

        // latest first.
        date_b.cmp(&date_a)
    });

    entries
}

fn create_jwk_set(
    certificate_entries: &Vec<DirEntry>,
) -> Result<JSONWebKeySet, OperationOutcomeError> {
    let mut jsonweb_key_set = JSONWebKeySet { keys: vec![] };

    for certification_entry in certificate_entries.iter() {
        let cert_path = certification_entry.path();
        let rsa_private =
            RsaPrivateKey::from_pkcs1_pem(&std::fs::read_to_string(cert_path).unwrap()).unwrap();
        let rsa_public_key = rsa_private.to_public_key();

        let mut hasher = Sha1::new();
        hasher.update(rsa_public_key.to_pkcs1_der().unwrap().as_bytes());
        let x5t = hasher.finalize();

        jsonweb_key_set.keys.push(JSONWebKey {
            kid: derive_kid(cert_path),
            alg: JSONWebKeyAlgorithm::RS256,
            kty: JSONWebKeyType::RSA,
            e: URL_SAFE_NO_PAD.encode(&rsa_public_key.e().clone().to_bytes_be()),
            n: URL_SAFE_NO_PAD.encode(&rsa_public_key.n().clone().to_bytes_be()),
            x5t: Some(URL_SAFE_NO_PAD.encode(&x5t)),
        });
    }

    Ok(jsonweb_key_set)
}

fn create_decoding_keys(
    certificate_entries: &Vec<DirEntry>,
) -> Result<Vec<DecodingKey>, OperationOutcomeError> {
    let mut decoding_keys = vec![];

    for certification_entry in certificate_entries.iter() {
        let cert_path = certification_entry.path();
        let rsa_private =
            RsaPrivateKey::from_pkcs1_pem(&std::fs::read_to_string(cert_path).unwrap()).unwrap();

        let rsa_public_key = rsa_private.to_public_key();

        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(
            rsa_public_key
                .to_pkcs1_pem(LineEnding::default())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();

        decoding_keys.push(DecodingKey {
            kid: derive_kid(cert_path),
            decoding_key,
        });
    }

    Ok(decoding_keys)
}

/// Latest key is first. this is set by date_b.cmp(&date_a) in get_sorted_private_cert_paths
fn get_encoding_keys(
    certificate_entries: &Vec<DirEntry>,
) -> Result<Vec<EncodingKey>, OperationOutcomeError> {
    let mut encoding_keys = vec![];

    for certification_entry in certificate_entries.iter() {
        let cert_path = certification_entry.path();
        let encoding_key =
            jsonwebtoken::EncodingKey::from_rsa_pem(&std::fs::read(cert_path).unwrap()).unwrap();

        encoding_keys.push(EncodingKey {
            kid: derive_kid(cert_path),
            encoding_key,
        });
    }

    Ok(encoding_keys)
}

fn create_certifications_if_needed(
    config: &dyn Config<ServerEnvironmentVariables>,
) -> Result<(), OperationOutcomeError> {
    let certificate_dir = config
        .get(ServerEnvironmentVariables::CertificationDir)
        .unwrap();
    let cert_dir: &Path = Path::new(&certificate_dir);

    let private_key_files = get_sorted_private_cert_paths(config);

    // If no private key than write.
    if private_key_files.is_empty() {
        let mut rng = OsRng;
        let bits: usize = 2048;

        // Use rfc 3339 format for date. Same as time_rotating.id.
        let date = chrono::Utc::now();
        let date2 = date + chrono::Days::new(5);

        let private_key_file_name1 = format!("k1_{}.pem", date.to_rfc3339());
        let private_key_file_name2 = format!("k2_{}.pem", date2.to_rfc3339());

        let priv_key1 = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let priv_key2 = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");

        std::fs::create_dir_all(cert_dir).unwrap();
        std::fs::write(
            cert_dir.join(private_key_file_name1),
            priv_key1.to_pkcs1_pem(LineEnding::default()).unwrap(),
        )
        .map_err(|e| OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string()))?;
        std::fs::write(
            cert_dir.join(private_key_file_name2),
            priv_key2.to_pkcs1_pem(LineEnding::default()).unwrap(),
        )
        .map_err(|e| OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string()))?;
    }

    Ok(())
}

pub struct LocalCertifications {
    decoding_key: Arc<Vec<DecodingKey>>,
    encoding_keys: Arc<Vec<EncodingKey>>,
    jwk_set: Arc<JSONWebKeySet>,
}

impl LocalCertifications {
    pub fn new(
        config: &dyn Config<ServerEnvironmentVariables>,
    ) -> Result<Self, OperationOutcomeError> {
        create_certifications_if_needed(config)?;

        let private_certificate_entries = get_sorted_private_cert_paths(config);

        Ok(LocalCertifications {
            decoding_key: Arc::new(create_decoding_keys(&private_certificate_entries)?),
            encoding_keys: Arc::new(get_encoding_keys(&private_certificate_entries)?),
            jwk_set: Arc::new(create_jwk_set(&private_certificate_entries)?),
        })
    }
}

impl CertificationProvider for LocalCertifications {
    fn decoding_key<'a>(&'a self, kid: &str) -> Result<&'a DecodingKey, OperationOutcomeError> {
        self.decoding_key
            .iter()
            .find(|d| d.kid == kid)
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    format!("No decoding key found for kid: '{}'", kid),
                )
            })
    }

    fn encoding_key<'a>(&'a self) -> Result<&'a EncodingKey, OperationOutcomeError> {
        self.encoding_keys.first().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                "No encoding key available".to_string(),
            )
        })
    }

    fn jwk_set(&self) -> Arc<JSONWebKeySet> {
        self.jwk_set.clone()
    }
}
