//! Message command implementations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use crate::commands::guards::{check_write_allowed, confirm_destructive};
use serde_json::json;
use std::collections::HashMap;

/// Post a message to a channel
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `text` - Message text
///
/// # Returns
/// * `Ok(ApiResponse)` with posted message information
/// * `Err(ApiError)` if the operation fails
pub async fn msg_post(
    client: &ApiClient,
    channel: String,
    text: String,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("text".to_string(), json!(text));

    client.call_method(ApiMethod::ChatPostMessage, params).await
}

/// Update a message
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `ts` - Message timestamp
/// * `text` - New message text
/// * `yes` - Skip confirmation prompt
///
/// # Returns
/// * `Ok(ApiResponse)` with updated message information
/// * `Err(ApiError)` if the operation fails
pub async fn msg_update(
    client: &ApiClient,
    channel: String,
    ts: String,
    text: String,
    yes: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;
    confirm_destructive(yes, "update this message")?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("ts".to_string(), json!(ts));
    params.insert("text".to_string(), json!(text));

    client.call_method(ApiMethod::ChatUpdate, params).await
}

/// Delete a message
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `ts` - Message timestamp
/// * `yes` - Skip confirmation prompt
///
/// # Returns
/// * `Ok(ApiResponse)` with deletion confirmation
/// * `Err(ApiError)` if the operation fails
pub async fn msg_delete(
    client: &ApiClient,
    channel: String,
    ts: String,
    yes: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;
    confirm_destructive(yes, "delete this message")?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("ts".to_string(), json!(ts));

    client.call_method(ApiMethod::ChatDelete, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_msg_post_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = msg_post(&client, "C123456".to_string(), "test message".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[tokio::test]
    async fn test_msg_update_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = msg_update(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            "updated text".to_string(),
            true,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[tokio::test]
    async fn test_msg_delete_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = msg_delete(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            true,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }
}
