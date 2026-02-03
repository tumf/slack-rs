//! PKCE (Proof Key for Code Exchange) implementation
//!
//! Provides functions to generate:
//! - Code verifier: random string
//! - Code challenge: SHA-256 hash of verifier, base64url encoded
//! - State: random string for CSRF protection

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

/// Generate a cryptographically secure random string
fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            if idx < 26 {
                (b'A' + idx) as char
            } else if idx < 52 {
                (b'a' + (idx - 26)) as char
            } else {
                (b'0' + (idx - 52)) as char
            }
        })
        .collect()
}

/// Generate PKCE code verifier and code challenge
///
/// Returns (code_verifier, code_challenge)
pub fn generate_pkce() -> (String, String) {
    let code_verifier = generate_random_string(128);
    let code_challenge = generate_code_challenge(&code_verifier);
    (code_verifier, code_challenge)
}

/// Generate code challenge from code verifier using SHA-256
fn generate_code_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate a random state string for CSRF protection
pub fn generate_state() -> String {
    generate_random_string(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pkce() {
        let (verifier, challenge) = generate_pkce();

        // Verifier should be 128 characters
        assert_eq!(verifier.len(), 128);

        // Challenge should not be empty
        assert!(!challenge.is_empty());

        // Verify that the challenge matches the verifier
        let expected_challenge = generate_code_challenge(&verifier);
        assert_eq!(challenge, expected_challenge);
    }

    #[test]
    fn test_generate_pkce_uniqueness() {
        let (verifier1, challenge1) = generate_pkce();
        let (verifier2, challenge2) = generate_pkce();

        // Each generation should produce unique values
        assert_ne!(verifier1, verifier2);
        assert_ne!(challenge1, challenge2);
    }

    #[test]
    fn test_generate_state() {
        let state = generate_state();

        // State should be 32 characters
        assert_eq!(state.len(), 32);

        // Should only contain alphanumeric characters
        assert!(state.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_state_uniqueness() {
        let state1 = generate_state();
        let state2 = generate_state();

        // Each generation should produce unique values
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_code_challenge_deterministic() {
        let verifier = "test_verifier_12345";
        let challenge1 = generate_code_challenge(verifier);
        let challenge2 = generate_code_challenge(verifier);

        // Same verifier should always produce the same challenge
        assert_eq!(challenge1, challenge2);
    }
}
