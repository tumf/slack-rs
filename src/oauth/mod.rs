//! OAuth 2.0 authentication flow with PKCE support
//!
//! This module provides OAuth authentication functionality for Slack CLI:
//! - PKCE (Proof Key for Code Exchange) generation
//! - State generation and verification for CSRF protection
//! - Authorization URL generation
//! - Token exchange with oauth.v2.access
//! - Local callback server for receiving authorization codes
//! - Callback port resolution from environment variables

pub mod pkce;
pub mod port;
pub mod server;
pub mod types;

pub use pkce::{generate_pkce, generate_state};
pub use port::resolve_callback_port;
pub use server::run_callback_server;
pub use types::{OAuthConfig, OAuthError, OAuthResponse};

use reqwest::Client;
use std::collections::HashMap;

/// Exchanges an authorization code for an access token
///
/// # Arguments
/// * `config` - OAuth configuration including client_id, client_secret, and redirect_uri
/// * `code` - Authorization code received from callback
/// * `code_verifier` - PKCE code verifier
/// * `base_url` - Optional base URL for testing (defaults to Slack's OAuth endpoint)
pub async fn exchange_code(
    config: &OAuthConfig,
    code: &str,
    code_verifier: &str,
    base_url: Option<&str>,
) -> Result<OAuthResponse, OAuthError> {
    let url = format!(
        "{}/oauth.v2.access",
        base_url.unwrap_or("https://slack.com/api")
    );

    let mut params = HashMap::new();
    params.insert("client_id", config.client_id.as_str());
    params.insert("client_secret", config.client_secret.as_str());
    params.insert("code", code);
    params.insert("redirect_uri", config.redirect_uri.as_str());
    params.insert("code_verifier", code_verifier);

    let client = Client::new();
    let response = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| OAuthError::NetworkError(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| OAuthError::NetworkError(e.to_string()))?;

    if !status.is_success() {
        return Err(OAuthError::HttpError(status.as_u16(), body));
    }

    let oauth_response: OAuthResponse =
        serde_json::from_str(&body).map_err(|e| OAuthError::ParseError(e.to_string()))?;

    if !oauth_response.ok {
        return Err(OAuthError::SlackError(
            oauth_response
                .error
                .unwrap_or_else(|| "unknown".to_string()),
        ));
    }

    Ok(oauth_response)
}

/// Generates the full authorization URL
///
/// # Arguments
/// * `config` - OAuth configuration
/// * `code_challenge` - PKCE code challenge
/// * `state` - CSRF protection state
pub fn build_authorization_url(
    config: &OAuthConfig,
    code_challenge: &str,
    state: &str,
) -> Result<String, OAuthError> {
    let base_url = "https://slack.com/oauth/v2/authorize";
    let mut url = url::Url::parse(base_url).map_err(|e| OAuthError::ParseError(e.to_string()))?;

    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("scope", &config.scopes.join(","))
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state);

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_authorization_url() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:8765/callback".to_string(),
            scopes: vec!["chat:write".to_string(), "users:read".to_string()],
        };

        let code_challenge = "test_challenge";
        let state = "test_state";

        let url = build_authorization_url(&config, code_challenge, state).unwrap();

        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("scope=chat%3Awrite%2Cusers%3Aread"));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8765%2Fcallback"));
        assert!(url.contains("code_challenge=test_challenge"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=test_state"));
    }

    #[tokio::test]
    async fn test_exchange_code_invalid_base_url() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:8765/callback".to_string(),
            scopes: vec!["chat:write".to_string()],
        };

        // Using an invalid URL should result in a network error
        let result = exchange_code(
            &config,
            "test_code",
            "test_verifier",
            Some("http://invalid-url-that-does-not-exist"),
        )
        .await;

        assert!(result.is_err());
        match result {
            Err(OAuthError::NetworkError(_)) => {}
            _ => panic!("Expected NetworkError"),
        }
    }
}
