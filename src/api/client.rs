//! HTTP client for Slack API calls
//!
//! This module provides a configurable HTTP client with:
//! - Configurable base URL (for testing with mock servers)
//! - Retry logic with exponential backoff
//! - Rate limit handling (429 + Retry-After)
//! - Support for both wrapper commands and generic API calls

use reqwest::{Client, Method, Response, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

use super::guidance::format_error_guidance;
use super::types::{ApiMethod, ApiResponse};

/// API client errors (for wrapper commands)
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("JSON serialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Slack API error: {0}")]
    SlackError(String),

    #[allow(dead_code)]
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Write operation denied. Set SLACKCLI_ALLOW_WRITE=true to enable write operations")]
    WriteNotAllowed,

    #[error("Destructive operation cancelled")]
    OperationCancelled,

    #[error("Non-interactive mode error: {0}")]
    NonInteractiveError(String),
}

/// API client errors (for generic API calls)
#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Rate limit exceeded, retry after {0} seconds")]
    RateLimitExceeded(u64),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, ApiClientError>;

/// Configuration for the API client
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    /// Base URL for API calls (default: https://slack.com/api)
    pub base_url: String,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,

    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://slack.com/api".to_string(),
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 32000,
        }
    }
}

/// Slack API client
///
/// Supports both:
/// - Wrapper commands via `call_method()` with `ApiMethod` enum
/// - Generic API calls via `call()` with arbitrary endpoints
pub struct ApiClient {
    client: Client,
    pub(crate) token: Option<String>,
    config: ApiClientConfig,
}

impl ApiClient {
    /// Create a new API client with default configuration (for generic API calls)
    pub fn new() -> Self {
        Self::with_config(ApiClientConfig::default())
    }

    /// Create a new API client with a token (for wrapper commands)
    pub fn with_token(token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            token: Some(token),
            config: ApiClientConfig::default(),
        }
    }

    /// Create a new API client with custom configuration
    pub fn with_config(config: ApiClientConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token: None,
            config,
        }
    }

    /// Create a new API client with custom base URL (for testing)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn new_with_base_url(token: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            token: Some(token),
            config: ApiClientConfig {
                base_url,
                ..Default::default()
            },
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Call a Slack API method using the ApiMethod enum (for wrapper commands)
    pub async fn call_method(
        &self,
        method: ApiMethod,
        params: HashMap<String, Value>,
    ) -> std::result::Result<ApiResponse, ApiError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| ApiError::SlackError("No token configured".to_string()))?;

        let url = format!("{}/{}", self.config.base_url, method.as_str());

        let response = if method.uses_get_method() {
            // Use GET request with query parameters
            let mut query_params = vec![];
            for (key, value) in params {
                let value_str = match value {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => serde_json::to_string(&value).unwrap_or_default(),
                };
                query_params.push((key, value_str));
            }

            self.client
                .get(&url)
                .bearer_auth(token)
                .query(&query_params)
                .send()
                .await?
        } else {
            // Use POST request with JSON body
            self.client
                .post(&url)
                .bearer_auth(token)
                .json(&params)
                .send()
                .await?
        };

        let response_json: ApiResponse = response.json().await?;

        if !response_json.ok {
            let error_code = response_json.error.as_deref().unwrap_or("Unknown error");

            // Display error guidance if available
            if let Some(guidance) = format_error_guidance(error_code) {
                eprintln!("{}", guidance);
            }

            return Err(ApiError::SlackError(error_code.to_string()));
        }

        Ok(response_json)
    }

    /// Make an API call with automatic retry logic (for generic API calls)
    pub async fn call(
        &self,
        method: Method,
        endpoint: &str,
        token: &str,
        body: RequestBody,
    ) -> Result<Response> {
        let url = format!("{}/{}", self.config.base_url, endpoint);
        let mut attempt = 0;

        loop {
            let response = self.execute_request(&url, &method, token, &body).await?;

            // Check for rate limiting
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                // Extract Retry-After header
                let retry_after = self.extract_retry_after(&response);

                if attempt >= self.config.max_retries {
                    return Err(ApiClientError::RateLimitExceeded(retry_after));
                }

                // Wait for the specified duration
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                attempt += 1;
                continue;
            }

            // For other errors, apply exponential backoff
            if !response.status().is_success() && attempt < self.config.max_retries {
                let backoff = self.calculate_backoff(attempt);
                tokio::time::sleep(backoff).await;
                attempt += 1;
                continue;
            }

            return Ok(response);
        }
    }

    /// Execute a single HTTP request
    async fn execute_request(
        &self,
        url: &str,
        method: &Method,
        token: &str,
        body: &RequestBody,
    ) -> Result<Response> {
        let mut request = self.client.request(method.clone(), url);

        // Add authorization header
        request = request.header("Authorization", format!("Bearer {}", token));

        // Add body based on type
        match body {
            RequestBody::Form(params) => {
                request = request
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .form(params);
            }
            RequestBody::Json(json) => {
                request = request
                    .header("Content-Type", "application/json")
                    .json(json);
            }
            RequestBody::None => {}
        }

        let response = request.send().await?;
        Ok(response)
    }

    /// Extract Retry-After header value
    fn extract_retry_after(&self, response: &Response) -> u64 {
        response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60) // Default to 60 seconds if not specified
    }

    /// Calculate exponential backoff with jitter
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base = self.config.initial_backoff_ms;
        let max = self.config.max_backoff_ms;

        // Exponential backoff: base * 2^attempt
        let backoff = base * 2_u64.pow(attempt);
        let backoff = backoff.min(max);

        // Add jitter (±25%)
        let jitter = (backoff as f64 * 0.25) as u64;
        let jitter = rand::random::<u64>() % (jitter * 2 + 1);
        let backoff = backoff
            .saturating_sub(jitter / 2)
            .saturating_add(jitter / 2);

        Duration::from_millis(backoff)
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Request body type
#[derive(Debug, Clone)]
pub enum RequestBody {
    Form(Vec<(String, String)>),
    Json(Value),
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_method_as_str() {
        assert_eq!(ApiMethod::SearchMessages.as_str(), "search.messages");
        assert_eq!(ApiMethod::ConversationsList.as_str(), "conversations.list");
        assert_eq!(
            ApiMethod::ConversationsHistory.as_str(),
            "conversations.history"
        );
        assert_eq!(ApiMethod::UsersInfo.as_str(), "users.info");
        assert_eq!(ApiMethod::ChatPostMessage.as_str(), "chat.postMessage");
        assert_eq!(ApiMethod::ChatUpdate.as_str(), "chat.update");
        assert_eq!(ApiMethod::ChatDelete.as_str(), "chat.delete");
        assert_eq!(ApiMethod::ReactionsAdd.as_str(), "reactions.add");
        assert_eq!(ApiMethod::ReactionsRemove.as_str(), "reactions.remove");
    }

    #[test]
    fn test_api_method_is_write() {
        assert!(!ApiMethod::SearchMessages.is_write());
        assert!(!ApiMethod::ConversationsList.is_write());
        assert!(!ApiMethod::ConversationsHistory.is_write());
        assert!(!ApiMethod::UsersInfo.is_write());
        assert!(ApiMethod::ChatPostMessage.is_write());
        assert!(ApiMethod::ChatUpdate.is_write());
        assert!(ApiMethod::ChatDelete.is_write());
        assert!(ApiMethod::ReactionsAdd.is_write());
        assert!(ApiMethod::ReactionsRemove.is_write());
    }

    #[test]
    fn test_api_method_is_destructive() {
        assert!(!ApiMethod::SearchMessages.is_destructive());
        assert!(!ApiMethod::ConversationsList.is_destructive());
        assert!(!ApiMethod::ConversationsHistory.is_destructive());
        assert!(!ApiMethod::UsersInfo.is_destructive());
        assert!(!ApiMethod::ChatPostMessage.is_destructive());
        assert!(ApiMethod::ChatUpdate.is_destructive());
        assert!(ApiMethod::ChatDelete.is_destructive());
        assert!(!ApiMethod::ReactionsAdd.is_destructive());
        assert!(ApiMethod::ReactionsRemove.is_destructive());
    }

    #[test]
    fn test_api_method_uses_get() {
        // GET methods
        assert!(ApiMethod::SearchMessages.uses_get_method());
        assert!(ApiMethod::ConversationsList.uses_get_method());
        assert!(ApiMethod::ConversationsHistory.uses_get_method());
        assert!(ApiMethod::UsersInfo.uses_get_method());
        assert!(ApiMethod::UsersList.uses_get_method());

        // POST methods
        assert!(!ApiMethod::ChatPostMessage.uses_get_method());
        assert!(!ApiMethod::ChatUpdate.uses_get_method());
        assert!(!ApiMethod::ChatDelete.uses_get_method());
        assert!(!ApiMethod::ReactionsAdd.uses_get_method());
        assert!(!ApiMethod::ReactionsRemove.uses_get_method());
    }

    #[test]
    fn test_api_client_config_default() {
        let config = ApiClientConfig::default();
        assert_eq!(config.base_url, "https://slack.com/api");
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 1000);
        assert_eq!(config.max_backoff_ms, 32000);
    }

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new();
        assert_eq!(client.base_url(), "https://slack.com/api");
    }

    #[test]
    fn test_api_client_custom_config() {
        let config = ApiClientConfig {
            base_url: "https://test.example.com".to_string(),
            max_retries: 5,
            initial_backoff_ms: 500,
            max_backoff_ms: 10000,
        };

        let client = ApiClient::with_config(config.clone());
        assert_eq!(client.base_url(), "https://test.example.com");
        assert_eq!(client.config.max_retries, 5);
    }
}
