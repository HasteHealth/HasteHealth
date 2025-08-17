use oxidized_config::{Config, ConfigType, get_config};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use rand::rngs::OsRng;
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
};
use std::{path::Path, sync::LazyLock};

static PRIVATE_KEY_FILENAME: &str = "private_key.pem";
static PUBLIC_KEY_FILENAME: &str = "public_key.pem";

pub fn create_certifications(config: &Box<dyn Config>) -> Result<(), OperationOutcomeError> {
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
        .map_err(|e| {
            OperationOutcomeError::fatal(OperationOutcomeCodes::Exception, e.to_string())
        })?;

        std::fs::write(
            public_key_file,
            pub_key.to_pkcs1_pem(LineEnding::default()).unwrap(),
        )
        .map_err(|e| {
            OperationOutcomeError::fatal(OperationOutcomeCodes::Exception, e.to_string())
        })?;
    }

    Ok(())
}

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

pub fn decoding_key() -> &'static jsonwebtoken::DecodingKey {
    &*DECODING_KEY
}

pub fn encoding_key() -> &'static jsonwebtoken::EncodingKey {
    &*ENCODING_KEY
}
