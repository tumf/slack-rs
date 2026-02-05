//! Reaction command implementations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use crate::commands::guards::{check_write_allowed, confirm_destructive_with_hint};
use serde_json::json;
use std::collections::HashMap;

/// Add a reaction to a message
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `timestamp` - Message timestamp
/// * `name` - Emoji name (without colons, e.g., "thumbsup")
///
/// # Returns
/// * `Ok(ApiResponse)` with reaction confirmation
/// * `Err(ApiError)` if the operation fails
pub async fn react_add(
    client: &ApiClient,
    channel: String,
    timestamp: String,
    name: String,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("timestamp".to_string(), json!(timestamp));
    params.insert("name".to_string(), json!(name));

    client.call_method(ApiMethod::ReactionsAdd, params).await
}

/// Remove a reaction from a message
///
/// # Arguments
/// * `client` - API client
/// * `channel` - Channel ID
/// * `timestamp` - Message timestamp
/// * `name` - Emoji name (without colons, e.g., "thumbsup")
/// * `yes` - Skip confirmation prompt
/// * `non_interactive` - Whether running in non-interactive mode
///
/// # Returns
/// * `Ok(ApiResponse)` with removal confirmation
/// * `Err(ApiError)` if the operation fails
pub async fn react_remove(
    client: &ApiClient,
    channel: String,
    timestamp: String,
    name: String,
    yes: bool,
    non_interactive: bool,
) -> Result<ApiResponse, ApiError> {
    check_write_allowed()?;

    // Build hint with example command for non-interactive mode
    let hint = format!(
        "Example: slack-rs react remove {} {} {} --yes",
        channel, timestamp, name
    );
    confirm_destructive_with_hint(yes, "remove this reaction", non_interactive, Some(&hint))?;

    let mut params = HashMap::new();
    params.insert("channel".to_string(), json!(channel));
    params.insert("timestamp".to_string(), json!(timestamp));
    params.insert("name".to_string(), json!(name));

    client.call_method(ApiMethod::ReactionsRemove, params).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_react_add_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = react_add(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            "thumbsup".to_string(),
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }

    #[tokio::test]
    #[serial(write_guard)]
    async fn test_react_remove_with_env_false() {
        std::env::set_var("SLACKCLI_ALLOW_WRITE", "false");
        let client = ApiClient::with_token("test_token".to_string());
        let result = react_remove(
            &client,
            "C123456".to_string(),
            "1234567890.123456".to_string(),
            "thumbsup".to_string(),
            true,
            false,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::WriteNotAllowed));
        std::env::remove_var("SLACKCLI_ALLOW_WRITE");
    }
}
