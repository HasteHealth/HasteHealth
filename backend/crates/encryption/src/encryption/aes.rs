use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, Generate, KeyInit, Nonce},
};
use haste_fhir_operation_error::OperationOutcomeError;

use crate::{error::EncryptionError, traits::Encryptor};

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// AES-256-GCM encryption. Output is `nonce || ciphertext`, with a fresh
/// random nonce generated for every `encrypt` call.
pub struct AesGcmEncryptor {
    cipher: Aes256Gcm,
}

impl AesGcmEncryptor {
    pub fn new(key: &[u8]) -> Result<Self, OperationOutcomeError> {
        let key_array = Key::<Aes256Gcm>::try_from(key)
            .map_err(|_| EncryptionError::InvalidKeyLength(KEY_LEN, key.len()))?;

        Ok(Self {
            cipher: Aes256Gcm::new(&key_array),
        })
    }
}

impl Encryptor for AesGcmEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, OperationOutcomeError> {
        let nonce = Nonce::<Aes256Gcm>::generate();

        let mut ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let mut output = nonce.to_vec();
        output.append(&mut ciphertext);
        Ok(output)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, OperationOutcomeError> {
        if ciphertext.len() < NONCE_LEN {
            return Err(EncryptionError::DecryptionFailed(
                "ciphertext shorter than nonce".to_string(),
            )
            .into());
        }

        let (nonce_bytes, ciphertext) = ciphertext.split_at(NONCE_LEN);
        let nonce = Nonce::<Aes256Gcm>::try_from(nonce_bytes)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_plaintext() {
        let encryptor = AesGcmEncryptor::new(&[7u8; KEY_LEN]).unwrap();
        let plaintext = b"totp-secret-material";

        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        assert_ne!(ciphertext, plaintext);

        let decrypted = encryptor.decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn rejects_wrong_key_length() {
        assert!(AesGcmEncryptor::new(&[0u8; 16]).is_err());
    }

    #[test]
    fn rejects_tampered_ciphertext() {
        let encryptor = AesGcmEncryptor::new(&[7u8; KEY_LEN]).unwrap();
        let mut ciphertext = encryptor.encrypt(b"totp-secret-material").unwrap();

        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;

        assert!(encryptor.decrypt(&ciphertext).is_err());
    }
}
