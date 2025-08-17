use oxidized_fhir_operation_error::OperationOutcomeError;
use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::{path::Path, sync::LazyLock};

pub fn create_certifications(dir: &Path) -> Result<(), OperationOutcomeError> {
    let mut rng = OsRng;
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    let private_key_file = dir.join("private_key.pem");
    let public_key_file = dir.join("public_key.pem");
}

pub fn get_decoding_key() -> jsonwebtoken::DecodingKey {
    let key = CERTIFICATES.public_key.clone();
    jsonwebtoken::DecodingKey::from_rsa_pem(&key.to_pem().unwrap()).unwrap()
}
