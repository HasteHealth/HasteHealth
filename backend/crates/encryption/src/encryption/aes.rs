use crate::{
    error::EncryptionError,
    traits::{EncryptionResult, Encryptor, Secret},
};
use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, Generate, KeyInit, Nonce, Payload},
};
use haste_fhir_operation_error::OperationOutcomeError;

const KEY_LEN: usize = 32;

/// AES-256-GCM encryption. Output is `nonce || ciphertext`, with a fresh
/// random nonce generated for every `encrypt` call.
pub struct AesGcmEncryptor {
    cipher: Aes256Gcm,
}

impl AesGcmEncryptor {
    /// Creates a new AES-256-GCM cipher from the provided key.
    ///
    /// # Errors
    ///
    /// Returns [`OperationOutcomeError`] if `key` is not exactly `KEY_LEN` bytes
    /// long.
    pub fn new(key: &[u8]) -> Result<Self, OperationOutcomeError> {
        let key_array = Key::<Aes256Gcm>::try_from(key)
            .map_err(|_| EncryptionError::InvalidKeyLength(KEY_LEN, key.len()))?;

        Ok(Self {
            cipher: Aes256Gcm::new(&key_array),
        })
    }

    /// A new random key for [`Self::new`], from the operating system's random
    /// number generator.
    #[must_use]
    pub fn generate_key() -> Secret {
        Secret::new(Key::<Aes256Gcm>::generate().to_vec())
    }
}

impl Encryptor for AesGcmEncryptor {
    fn encrypt_with_aad(
        &self,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<EncryptionResult, OperationOutcomeError> {
        let nonce = Nonce::<Aes256Gcm>::generate();

        let ciphertext = self
            .cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptionResult {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    fn decrypt_with_aad(
        &self,
        encrypted_result: &EncryptionResult,
        aad: &[u8],
    ) -> Result<Vec<u8>, OperationOutcomeError> {
        let nonce = Nonce::<Aes256Gcm>::try_from(encrypted_result.nonce.as_slice())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        let plaintext = self
            .cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: encrypted_result.ciphertext.as_slice(),
                    aad,
                },
            )
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
    fn generates_a_usable_key_that_differs_each_time() {
        let key = AesGcmEncryptor::generate_key();
        assert_eq!(key.expose_bytes().len(), KEY_LEN);
        assert_ne!(
            key.expose_bytes(),
            AesGcmEncryptor::generate_key().expose_bytes()
        );

        let encryptor = AesGcmEncryptor::new(key.expose_bytes()).unwrap();
        let result = encryptor.encrypt(b"totp-secret-material").unwrap();
        assert_eq!(encryptor.decrypt(&result).unwrap(), b"totp-secret-material");
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

    #[test]
    fn round_trips_plaintext_with_associated_data() {
        let encryptor = AesGcmEncryptor::new(&[7u8; KEY_LEN]).unwrap();
        let plaintext = b"totp-secret-material";

        let result = encryptor
            .encrypt_with_aad(plaintext, b"tenant-a/user-1")
            .unwrap();

        let decrypted = encryptor
            .decrypt_with_aad(&result, b"tenant-a/user-1")
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    /// A ciphertext copied to another owner's row does not decrypt there.
    #[test]
    fn rejects_other_associated_data() {
        let encryptor = AesGcmEncryptor::new(&[7u8; KEY_LEN]).unwrap();
        let result = encryptor
            .encrypt_with_aad(b"totp-secret-material", b"tenant-a/user-1")
            .unwrap();

        assert!(
            encryptor
                .decrypt_with_aad(&result, b"tenant-b/user-1")
                .is_err()
        );
        assert!(encryptor.decrypt(&result).is_err());
    }

    /// What `encrypt` wrote before associated data existed still decrypts.
    #[test]
    fn no_associated_data_is_empty_associated_data() {
        let encryptor = AesGcmEncryptor::new(&[7u8; KEY_LEN]).unwrap();
        let plaintext = b"totp-secret-material";

        let without = encryptor.encrypt(plaintext).unwrap();
        assert_eq!(
            encryptor.decrypt_with_aad(&without, &[]).unwrap(),
            plaintext
        );

        let empty = encryptor.encrypt_with_aad(plaintext, &[]).unwrap();
        assert_eq!(encryptor.decrypt(&empty).unwrap(), plaintext);
    }
}
