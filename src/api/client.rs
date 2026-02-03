//! Slack API client implementation

use super::types::{ApiMethod, ApiResponse};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// API client errors
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

    #[error("Write operation requires --allow-write flag")]
    WriteNotAllowed,

    #[error("Destructive operation cancelled")]
    OperationCancelled,
}

/// Slack API client
pub struct ApiClient {
    client: Client,
    token: String,
    base_url: String,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url: "https://slack.com/api".to_string(),
        }
    }

    /// Create a new API client with custom base URL (for testing)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn new_with_base_url(token: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url,
        }
    }

    /// Call a Slack API method
    pub async fn call(
        &self,
        method: ApiMethod,
        params: HashMap<String, Value>,
    ) -> Result<ApiResponse, ApiError> {
        let url = format!("{}/{}", self.base_url, method.as_str());

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&params)
            .send()
            .await?;

        let response_json: ApiResponse = response.json().await?;

        if !response_json.ok {
            return Err(ApiError::SlackError(
                response_json
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(response_json)
    }
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
}
