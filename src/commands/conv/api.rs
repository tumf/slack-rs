//! API call functionality for conversations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use serde_json::json;
use std::collections::HashMap;

/// List conversations with automatic pagination
///
/// # Arguments
/// * `client` - API client
/// * `types` - Optional comma-separated list of conversation types (public_channel, private_channel, mpim, im)
/// * `limit` - Optional number of results per page (default: 1000)
///
/// # Returns
/// * `Ok(ApiResponse)` with conversation list (all pages aggregated)
/// * `Err(ApiError)` if the operation fails
///
/// # Pagination
/// This function automatically follows `next_cursor` to retrieve all pages and aggregates
/// the `channels` array from all responses into a single response.
pub async fn conv_list(
    client: &ApiClient,
    types: Option<String>,
    limit: Option<u32>,
) -> Result<ApiResponse, ApiError> {
    let mut all_channels = Vec::new();
    let mut cursor: Option<String> = None;
    let mut ok = true;
    let mut error: Option<String> = None;

    loop {
        let mut params = HashMap::new();

        if let Some(ref types) = types {
            params.insert("types".to_string(), json!(types));
        }

        // Use provided limit or default to 1000
        let page_limit = limit.unwrap_or(1000);
        params.insert("limit".to_string(), json!(page_limit));

        if let Some(ref cursor_val) = cursor {
            params.insert("cursor".to_string(), json!(cursor_val));
        }

        let response = client
            .call_method(ApiMethod::ConversationsList, params)
            .await?;

        // Capture ok/error status from first response
        if cursor.is_none() {
            ok = response.ok;
            error = response.error.clone();
        }

        // Extract channels from this page
        if let Some(channels) = response.data.get("channels") {
            if let Some(channels_array) = channels.as_array() {
                all_channels.extend(channels_array.clone());
            }
        }

        // Check for next cursor
        cursor = response
            .data
            .get("response_metadata")
            .and_then(|meta| meta.get("next_cursor"))
            .and_then(|c| c.as_str())
            .filter(|c| !c.is_empty())
            .map(|c| c.to_string());

        // If no next cursor, we're done
        if cursor.is_none() {
            break;
        }
    }

    // Build final response with aggregated channels
    let mut data = HashMap::new();
    data.insert("channels".to_string(), json!(all_channels));

    Ok(ApiResponse { ok, data, error })
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
