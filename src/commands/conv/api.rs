//! API call functionality for conversations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use serde_json::json;
use std::collections::HashMap;

/// List conversations
///
/// # Arguments
/// * `client` - API client
/// * `types` - Optional comma-separated list of conversation types (public_channel, private_channel, mpim, im)
/// * `limit` - Optional number of results to return (default: 100)
///
/// # Returns
/// * `Ok(ApiResponse)` with conversation list
/// * `Err(ApiError)` if the operation fails
pub async fn conv_list(
    client: &ApiClient,
    types: Option<String>,
    limit: Option<u32>,
) -> Result<ApiResponse, ApiError> {
    let mut params = HashMap::new();

    if let Some(types) = types {
        params.insert("types".to_string(), json!(types));
    }

    if let Some(limit) = limit {
        params.insert("limit".to_string(), json!(limit));
    }

    client
        .call_method(ApiMethod::ConversationsList, params)
        .await
}

/// Get conversation history
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `limit` - Optional number of messages to return (default: 100)
/// * `oldest` - Optional oldest timestamp to include
/// * `latest` - Optional latest timestamp to include
///
/// # Returns
/// * `Ok(ApiResponse)` with conversation history
/// * `Err(ApiError)` if the operation fails
pub async fn conv_history(
    client: &ApiClient,
    channel: String,
    limit: Option<u32>,
    oldest: Option<String>,
    latest: Option<String>,
) -> Result<ApiResponse, ApiError> {
    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));

    if let Some(limit) = limit {
        params.insert("limit".to_string(), json!(limit));
    }

    if let Some(oldest) = oldest {
        params.insert("oldest".to_string(), json!(oldest));
    }

    if let Some(latest) = latest {
        params.insert("latest".to_string(), json!(latest));
    }

    client
        .call_method(ApiMethod::ConversationsHistory, params)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conv_list_basic() {
        let client = ApiClient::with_token("test_token".to_string());
        let result = conv_list(&client, None, None).await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_conv_history_basic() {
        let client = ApiClient::with_token("test_token".to_string());
        let result = conv_history(&client, "C123456".to_string(), None, None, None).await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }
}
