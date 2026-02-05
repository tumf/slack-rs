//! Message command implementations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use crate::commands::guards::{check_write_allowed, confirm_destructive_with_hint};
use serde_json::json;
use std::collections::HashMap;

/// Post a message to a channel
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `text` - Message text
/// * `thread_ts` - Optional thread timestamp to reply to
/// * `reply_broadcast` - Whether to broadcast thread reply to channel
///
/// # Returns
/// * `Ok(ApiResponse)` with posted message information
/// * `Err(ApiError)` if the operation fails
pub async fn msg_post(
    client: &ApiClient,
    channel: String,
    text: String,
    thread_ts: Option<String>,
    reply_broadcast: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("text".to_string(), json!(text));

    if let Some(ts) = thread_ts {
        params.insert("thread_ts".to_string(), json!(ts));
        if reply_broadcast {
            params.insert("reply_broadcast".to_string(), json!(true));
        }
    }

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
/// * `non_interactive` - Whether running in non-interactive mode
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
    non_interactive: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;

    // Build hint with example command for non-interactive mode
    let hint = format!(
        "Example: slack-rs msg update {} {} \"new text\" --yes",
        channel, ts
    );
    confirm_destructive_with_hint(yes, "update this message", non_interactive, Some(&hint))?;

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
/// * `non_interactive` - Whether running in non-interactive mode
///
/// # Returns
/// * `Ok(ApiResponse)` with deletion confirmation
/// * `Err(ApiError)` if the operation fails
pub async fn msg_delete(
    client: &ApiClient,
    channel: String,
    ts: String,
    yes: bool,
    non_interactive: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;

    // Build hint with example command for non-interactive mode
    let hint = format!("Example: slack-rs msg delete {} {} --yes", channel, ts);
    confirm_destructive_with_hint(yes, "delete this message", non_interactive, Some(&hint))?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("ts".to_string(), json!(ts));

    client.call_method(ApiMethod::ChatDelete, params).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_msg_post_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = msg_post(
            &client,
            "C123456".to_string(),
            "test message".to_string(),
            None,
            false,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_msg_update_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = msg_update(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            "updated text".to_string(),
            true,
            false,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_msg_delete_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = msg_delete(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            true,
            false,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }
}
