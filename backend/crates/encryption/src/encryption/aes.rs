use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, Generate, KeyInit, Nonce},
};
use haste_fhir_operation_error::OperationOutcomeError;

use crate::{
    error::EncryptionError,
    traits::{EncryptionResult, Encryptor},
};

const KEY_LEN: usize = 32;

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
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, OperationOutcomeError> {
        let nonce = Nonce::<Aes256Gcm>::generate();

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptionResult {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt(
        &self,
        encyrpted_result: &EncryptionResult,
    ) -> Result<Vec<u8>, OperationOutcomeError> {
        let nonce = Nonce::<Aes256Gcm>::try_from(encyrpted_result.nonce.as_slice())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        let plaintext = self
            .cipher
            .decrypt(&nonce, encyrpted_result.ciphertext.as_slice())
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
        assert_ne!(ciphertext.ciphertext, plaintext);

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
        let mut result = encryptor.encrypt(b"totp-secret-material").unwrap();

        let last = result.ciphertext.len() - 1;
        result.ciphertext[last] ^= 0xFF;

        assert!(encryptor.decrypt(&result).is_err());
    }
}
