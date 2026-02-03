//! OAuth types and configuration

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("OAuth configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("HTTP error {0}: {1}")]
    HttpError(u16, String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Slack API error: {0}")]
    SlackError(String),

    #[error("State mismatch: expected {expected}, got {actual}")]
    StateMismatch { expected: String, actual: String },

    #[error("Callback server error: {0}")]
    ServerError(String),

    #[error("Browser launch error: {0}")]
    #[allow(dead_code)]
    BrowserError(String),
}

/// OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthConfig {
    /// Validate that all required fields are set
    pub fn validate(&self) -> Result<(), OAuthError> {
        if self.client_id.is_empty() {
            return Err(OAuthError::ConfigError("client_id is required".to_string()));
        }
        if self.client_secret.is_empty() {
            return Err(OAuthError::ConfigError(
                "client_secret is required".to_string(),
            ));
        }
        if self.redirect_uri.is_empty() {
            return Err(OAuthError::ConfigError(
                "redirect_uri is required".to_string(),
            ));
        }
        if self.scopes.is_empty() {
            return Err(OAuthError::ConfigError("scopes are required".to_string()));
        }
        Ok(())
    }
}

/// OAuth response from Slack
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub ok: bool,
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub bot_user_id: Option<String>,
    pub app_id: Option<String>,
    pub team: Option<TeamInfo>,
    pub authed_user: Option<AuthedUser>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthedUser {
    pub id: String,
    pub scope: Option<String>,
    pub access_token: Option<String>,
    pub token_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config_validation_success() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scopes: vec!["chat:write".to_string()],
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_oauth_config_validation_empty_client_id() {
        let config = OAuthConfig {
            client_id: "".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scopes: vec!["chat:write".to_string()],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => assert!(msg.contains("client_id")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_oauth_config_validation_empty_client_secret() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scopes: vec!["chat:write".to_string()],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => assert!(msg.contains("client_secret")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_oauth_config_validation_empty_redirect_uri() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "".to_string(),
            scopes: vec!["chat:write".to_string()],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => assert!(msg.contains("redirect_uri")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_oauth_config_validation_empty_scopes() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            scopes: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(OAuthError::ConfigError(msg)) => assert!(msg.contains("scopes")),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_oauth_response_deserialization() {
        let json = r#"{
            "ok": true,
            "access_token": "xoxb-test-token",
            "token_type": "bot",
            "scope": "chat:write",
            "bot_user_id": "U123",
            "app_id": "A456",
            "team": {
                "id": "T789",
                "name": "Test Team"
            },
            "authed_user": {
                "id": "U012",
                "scope": "users:read",
                "access_token": "xoxp-test-token",
                "token_type": "user"
            }
        }"#;

        let response: OAuthResponse = serde_json::from_str(json).unwrap();
        assert!(response.ok);
        assert_eq!(response.access_token, Some("xoxb-test-token".to_string()));
        assert_eq!(response.team.as_ref().unwrap().id, "T789");
        assert_eq!(response.authed_user.as_ref().unwrap().id, "U012");
    }
}
