//! Cryptographic operations for export/import functionality
//!
//! Uses Argon2id for key derivation and AES-256-GCM for encryption

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::RngCore;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Invalid passphrase")]
    InvalidPassphrase,
}

pub type Result<T> = std::result::Result<T, CryptoError>;

/// KDF parameters for Argon2id
#[derive(Debug, Clone)]
pub struct KdfParams {
    pub salt: Vec<u8>,
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            salt: Vec::new(),
            memory_cost: 19456, // 19 MiB
            time_cost: 2,
            parallelism: 1,
        }
    }
}

/// Encrypted data with nonce
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

/// Derive encryption key from passphrase using Argon2id
pub fn derive_key(passphrase: &str, params: &KdfParams) -> Result<[u8; 32]> {
    if passphrase.is_empty() {
        return Err(CryptoError::InvalidPassphrase);
    }

    let argon2 = Argon2::default();

    // Convert salt to SaltString format
    let salt_string = SaltString::encode_b64(&params.salt)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    // Hash the password
    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt_string)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    // Extract the hash bytes (first 32 bytes)
    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| CryptoError::KeyDerivationFailed("No hash generated".to_string()))?;

    let hash_slice = hash_bytes.as_bytes();
    if hash_slice.len() < 32 {
        return Err(CryptoError::KeyDerivationFailed(
            "Hash too short".to_string(),
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_slice[..32]);
    Ok(key)
}

/// Generate random salt for KDF
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypt data with AES-256-GCM
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<EncryptedData> {
    let cipher = Aes256Gcm::new(key.into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok(EncryptedData {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(encrypted: &EncryptedData, key: &[u8; 32]) -> Result<Vec<u8>> {
    if encrypted.nonce.len() != 12 {
        return Err(CryptoError::DecryptionFailed(
            "Invalid nonce length".to_string(),
        ));
    }

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&encrypted.nonce);

    let plaintext = cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let passphrase = "test_password";
        let params = KdfParams {
            salt: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            memory_cost: 19456,
            time_cost: 2,
            parallelism: 1,
        };

        let key1 = derive_key(passphrase, &params).unwrap();
        let key2 = derive_key(passphrase, &params).unwrap();

        assert_eq!(
            key1, key2,
            "Same passphrase and salt should produce same key"
        );
    }

    #[test]
    fn test_derive_key_empty_passphrase() {
        let params = KdfParams::default();
        let result = derive_key("", &params);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::InvalidPassphrase
        ));
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let passphrase = "test_password";
        let plaintext = b"Hello, World!";

        let params = KdfParams {
            salt: generate_salt(),
            ..Default::default()
        };

        let key = derive_key(passphrase, &params).unwrap();

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let plaintext = b"Hello, World!";

        let params1 = KdfParams {
            salt: generate_salt(),
            ..Default::default()
        };
        let key1 = derive_key("password1", &params1).unwrap();

        let params2 = KdfParams {
            salt: generate_salt(),
            ..Default::default()
        };
        let key2 = derive_key("password2", &params2).unwrap();

        let encrypted = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::DecryptionFailed(_)
        ));
    }

    #[test]
    fn test_generate_salt_unique() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert_ne!(salt1, salt2, "Generated salts should be unique");
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = [0u8; 32];
        let plaintext = b"test";

        let encrypted1 = encrypt(plaintext, &key).unwrap();
        let encrypted2 = encrypt(plaintext, &key).unwrap();

        assert_ne!(
            encrypted1.nonce, encrypted2.nonce,
            "Nonces should be unique"
        );
    }
}
