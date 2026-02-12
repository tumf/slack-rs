//! Export/import file format definition
//!
//! Binary format:
//! - Magic bytes (8 bytes): "SLACKCLI"
//! - Format version (4 bytes, u32, big-endian)
//! - KDF params length (4 bytes, u32, big-endian)
//! - KDF params (variable length, JSON)
//! - Nonce length (4 bytes, u32, big-endian)
//! - Nonce (variable length)
//! - Ciphertext length (4 bytes, u32, big-endian)
//! - Ciphertext (variable length)

use crate::auth::crypto::{EncryptedData, KdfParams};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Invalid magic bytes")]
    InvalidMagic,
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, FormatError>;

const MAGIC: &[u8; 8] = b"SLACKCLI";
const CURRENT_VERSION: u32 = 1;

/// Profile data for export (includes token and optional OAuth credentials)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProfile {
    pub team_id: String,
    pub user_id: String,
    pub team_name: Option<String>,
    pub user_name: Option<String>,
    pub token: String,
    /// OAuth client ID (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// OAuth client secret (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// User token (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_token: Option<String>,
}

/// Export payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub format_version: u32,
    pub profiles: HashMap<String, ExportProfile>,

    // Allow unknown fields for forward compatibility
    #[serde(flatten)]
    #[serde(default)]
    pub unknown_fields: HashMap<String, serde_json::Value>,
}

impl ExportPayload {
    pub fn new() -> Self {
        Self {
            format_version: CURRENT_VERSION,
            profiles: HashMap::new(),
            unknown_fields: HashMap::new(),
        }
    }
}

impl Default for ExportPayload {
    fn default() -> Self {
        Self::new()
    }
}

/// Encoded export data (binary format)
#[derive(Debug, Clone)]
pub struct EncodedExport {
    pub kdf_params: KdfParams,
    pub encrypted_data: EncryptedData,
}

/// Encode export payload to binary format
pub fn encode_export(
    payload: &ExportPayload,
    encrypted: &EncryptedData,
    kdf_params: &KdfParams,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();

    // Magic bytes
    output.extend_from_slice(MAGIC);

    // Format version
    output.extend_from_slice(&payload.format_version.to_be_bytes());

    // KDF params as JSON
    let kdf_json = serde_json::json!({
        "salt": BASE64.encode(&kdf_params.salt),
        "memory_cost": kdf_params.memory_cost,
        "time_cost": kdf_params.time_cost,
        "parallelism": kdf_params.parallelism,
    });
    let kdf_bytes = serde_json::to_vec(&kdf_json)?;
    output.extend_from_slice(&(kdf_bytes.len() as u32).to_be_bytes());
    output.extend_from_slice(&kdf_bytes);

    // Nonce
    output.extend_from_slice(&(encrypted.nonce.len() as u32).to_be_bytes());
    output.extend_from_slice(&encrypted.nonce);

    // Ciphertext
    output.extend_from_slice(&(encrypted.ciphertext.len() as u32).to_be_bytes());
    output.extend_from_slice(&encrypted.ciphertext);

    Ok(output)
}

/// Decode export from binary format
pub fn decode_export(data: &[u8]) -> Result<EncodedExport> {
    if data.len() < 8 {
        return Err(FormatError::InvalidFormat("File too small".to_string()));
    }

    let mut cursor = 0;

    // Check magic
    if &data[cursor..cursor + 8] != MAGIC {
        return Err(FormatError::InvalidMagic);
    }
    cursor += 8;

    // Read format version
    if data.len() < cursor + 4 {
        return Err(FormatError::InvalidFormat(
            "Missing format version".to_string(),
        ));
    }
    let version = u32::from_be_bytes([
        data[cursor],
        data[cursor + 1],
        data[cursor + 2],
        data[cursor + 3],
    ]);
    cursor += 4;

    if version != CURRENT_VERSION {
        return Err(FormatError::UnsupportedVersion(version));
    }

    // Read KDF params
    if data.len() < cursor + 4 {
        return Err(FormatError::InvalidFormat(
            "Missing KDF params length".to_string(),
        ));
    }
    let kdf_len = u32::from_be_bytes([
        data[cursor],
        data[cursor + 1],
        data[cursor + 2],
        data[cursor + 3],
    ]) as usize;
    cursor += 4;

    if data.len() < cursor + kdf_len {
        return Err(FormatError::InvalidFormat(
            "Missing KDF params data".to_string(),
        ));
    }
    let kdf_json: serde_json::Value = serde_json::from_slice(&data[cursor..cursor + kdf_len])?;
    cursor += kdf_len;

    let salt = BASE64
        .decode(
            kdf_json["salt"]
                .as_str()
                .ok_or_else(|| FormatError::InvalidFormat("Missing salt".to_string()))?,
        )
        .map_err(|e| FormatError::InvalidFormat(format!("Invalid salt: {}", e)))?;

    let kdf_params = KdfParams {
        salt,
        memory_cost: kdf_json["memory_cost"]
            .as_u64()
            .ok_or_else(|| FormatError::InvalidFormat("Missing memory_cost".to_string()))?
            as u32,
        time_cost: kdf_json["time_cost"]
            .as_u64()
            .ok_or_else(|| FormatError::InvalidFormat("Missing time_cost".to_string()))?
            as u32,
        parallelism: kdf_json["parallelism"]
            .as_u64()
            .ok_or_else(|| FormatError::InvalidFormat("Missing parallelism".to_string()))?
            as u32,
    };

    // Read nonce
    if data.len() < cursor + 4 {
        return Err(FormatError::InvalidFormat(
            "Missing nonce length".to_string(),
        ));
    }
    let nonce_len = u32::from_be_bytes([
        data[cursor],
        data[cursor + 1],
        data[cursor + 2],
        data[cursor + 3],
    ]) as usize;
    cursor += 4;

    if data.len() < cursor + nonce_len {
        return Err(FormatError::InvalidFormat("Missing nonce data".to_string()));
    }
    let nonce = data[cursor..cursor + nonce_len].to_vec();
    cursor += nonce_len;

    // Read ciphertext
    if data.len() < cursor + 4 {
        return Err(FormatError::InvalidFormat(
            "Missing ciphertext length".to_string(),
        ));
    }
    let ciphertext_len = u32::from_be_bytes([
        data[cursor],
        data[cursor + 1],
        data[cursor + 2],
        data[cursor + 3],
    ]) as usize;
    cursor += 4;

    if data.len() < cursor + ciphertext_len {
        return Err(FormatError::InvalidFormat(
            "Missing ciphertext data".to_string(),
        ));
    }
    let ciphertext = data[cursor..cursor + ciphertext_len].to_vec();

    Ok(EncodedExport {
        kdf_params,
        encrypted_data: EncryptedData { nonce, ciphertext },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::crypto;

    #[test]
    fn test_export_payload_serialization() {
        let mut payload = ExportPayload::new();
        payload.profiles.insert(
            "default".to_string(),
            ExportProfile {
                team_id: "T123".to_string(),
                user_id: "U456".to_string(),
                team_name: Some("Test Team".to_string()),
                user_name: Some("Test User".to_string()),
                token: "xoxb-test-token".to_string(),
                client_id: None,
                client_secret: None,
                user_token: None,
            },
        );

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: ExportPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(payload.format_version, deserialized.format_version);
        assert_eq!(payload.profiles.len(), deserialized.profiles.len());
    }

    #[test]
    fn test_export_payload_unknown_fields() {
        let json = r#"{
            "format_version": 1,
            "profiles": {},
            "future_field": "some_value"
        }"#;

        let payload: ExportPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.format_version, 1);
        assert!(payload.unknown_fields.contains_key("future_field"));
    }

    #[test]
    fn test_encode_decode_round_trip() {
        let payload = ExportPayload::new();
        let passphrase = "test_password";

        // Create KDF params
        let kdf_params = KdfParams {
            salt: crypto::generate_salt(),
            ..Default::default()
        };

        // Encrypt payload
        let payload_json = serde_json::to_vec(&payload).unwrap();
        let key = crypto::derive_key(passphrase, &kdf_params).unwrap();
        let encrypted = crypto::encrypt(&payload_json, &key).unwrap();

        // Encode to binary
        let encoded = encode_export(&payload, &encrypted, &kdf_params).unwrap();

        // Decode from binary
        let decoded = decode_export(&encoded).unwrap();

        // Verify KDF params match
        assert_eq!(kdf_params.salt, decoded.kdf_params.salt);
        assert_eq!(kdf_params.memory_cost, decoded.kdf_params.memory_cost);
        assert_eq!(kdf_params.time_cost, decoded.kdf_params.time_cost);
        assert_eq!(kdf_params.parallelism, decoded.kdf_params.parallelism);

        // Verify encrypted data matches
        assert_eq!(encrypted.nonce, decoded.encrypted_data.nonce);
        assert_eq!(encrypted.ciphertext, decoded.encrypted_data.ciphertext);
    }

    #[test]
    fn test_decode_invalid_magic() {
        let data = b"INVALID_DATA";
        let result = decode_export(data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FormatError::InvalidMagic));
    }

    #[test]
    fn test_decode_unsupported_version() {
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC);
        data.extend_from_slice(&999u32.to_be_bytes()); // Unsupported version

        let result = decode_export(&data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FormatError::UnsupportedVersion(999)
        ));
    }
}
