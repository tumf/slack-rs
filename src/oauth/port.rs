//! OAuth callback port resolution
//!
//! Provides helpers to resolve the OAuth callback server port
//! from environment variables with a default fallback.

use super::types::OAuthError;

/// Default port for OAuth callback server
pub const DEFAULT_OAUTH_PORT: u16 = 8765;

/// Environment variable name for overriding the OAuth callback port
pub const OAUTH_PORT_ENV: &str = "SLACK_OAUTH_PORT";

/// Resolves the OAuth callback port from environment or uses default
///
/// The port is resolved in the following order:
/// 1. `SLACK_OAUTH_PORT` environment variable (if set and valid)
/// 2. Default port 8765
///
/// # Returns
/// * `Ok(u16)` - The resolved port number
/// * `Err(OAuthError)` - If `SLACK_OAUTH_PORT` is set but invalid
///
/// # Examples
/// ```
/// use slack_rs::oauth::resolve_callback_port;
///
/// // With default (no env var set)
/// let port = resolve_callback_port().unwrap();
/// assert_eq!(port, 8765);
///
/// // With env var set
/// std::env::set_var("SLACK_OAUTH_PORT", "9000");
/// let port = resolve_callback_port().unwrap();
/// assert_eq!(port, 9000);
/// ```
pub fn resolve_callback_port() -> Result<u16, OAuthError> {
    match std::env::var(OAUTH_PORT_ENV) {
        Ok(port_str) => {
            let port_str = port_str.trim();
            if port_str.is_empty() {
                // Empty or whitespace-only env var is a configuration error
                return Err(OAuthError::ConfigError(format!(
                    "{} is set but empty or contains only whitespace",
                    OAUTH_PORT_ENV
                )));
            }

            // Parse the port
            match port_str.parse::<u16>() {
                Ok(port) => {
                    // Validate port range (1-65535, but 0 is invalid for binding)
                    if port == 0 {
                        return Err(OAuthError::ConfigError(format!(
                            "Invalid port in {}: port must be between 1 and 65535",
                            OAUTH_PORT_ENV
                        )));
                    }
                    Ok(port)
                }
                Err(_) => Err(OAuthError::ConfigError(format!(
                    "Invalid port in {}: '{}' is not a valid port number",
                    OAUTH_PORT_ENV, port_str
                ))),
            }
        }
        Err(_) => {
            // Environment variable not set, use default
            Ok(DEFAULT_OAUTH_PORT)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use serial_test to avoid environment variable conflicts in parallel tests
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_resolve_callback_port_default() {
        // Clear env var to test default
        std::env::remove_var(OAUTH_PORT_ENV);
        let port = resolve_callback_port().unwrap();
        assert_eq!(port, DEFAULT_OAUTH_PORT);
        assert_eq!(port, 8765);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_from_env() {
        // Clear env first to avoid interference from other tests
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "9000");
        let port = resolve_callback_port().unwrap();
        assert_eq!(port, 9000);
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_from_env_edge_cases() {
        std::env::remove_var(OAUTH_PORT_ENV);
        // Test port 1 (minimum valid)
        std::env::set_var(OAUTH_PORT_ENV, "1");
        let port = resolve_callback_port().unwrap();
        assert_eq!(port, 1);

        // Test port 65535 (maximum valid)
        std::env::set_var(OAUTH_PORT_ENV, "65535");
        let port = resolve_callback_port().unwrap();
        assert_eq!(port, 65535);

        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_empty_env() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("is set but empty"));
            }
            _ => panic!("Expected ConfigError for empty env var"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_whitespace_around_number() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "  9123  ");
        let port = resolve_callback_port().unwrap();
        assert_eq!(port, 9123);
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_whitespace_only() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "   ");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("is set but empty"));
            }
            _ => panic!("Expected ConfigError for whitespace-only env var"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_invalid_zero() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "0");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("port must be between 1 and 65535"));
            }
            _ => panic!("Expected ConfigError for port 0"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_invalid_negative() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "-1");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("is not a valid port number"));
            }
            _ => panic!("Expected ConfigError for negative port"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_invalid_too_large() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "65536");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("is not a valid port number"));
            }
            _ => panic!("Expected ConfigError for port > 65535"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_invalid_non_numeric() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "not-a-port");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("is not a valid port number"));
            }
            _ => panic!("Expected ConfigError for non-numeric port"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }

    #[test]
    #[serial]
    fn test_resolve_callback_port_invalid_float() {
        std::env::remove_var(OAUTH_PORT_ENV);
        std::env::set_var(OAUTH_PORT_ENV, "8765.5");
        let result = resolve_callback_port();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => {
                assert!(msg.contains("is not a valid port number"));
            }
            _ => panic!("Expected ConfigError for float port"),
        }
        std::env::remove_var(OAUTH_PORT_ENV);
    }
}
