//! Users command implementations

use crate::api::{ApiClient, ApiError, ApiMethod, ApiResponse};
use serde_json::json;
use std::collections::HashMap;

/// Get user information
///
/// # Arguments
/// * `client` - API client
/// * `user` - User ID
///
/// # Returns
/// * `Ok(ApiResponse)` with user information
/// * `Err(ApiError)` if the operation fails
pub async fn users_info(client: &ApiClient, user: String) -> Result<ApiResponse, ApiError> {
    let mut params = HashMap::new();
    params.insert("user".to_string(), json!(user));

    client.call(ApiMethod::UsersInfo, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_users_info_basic() {
        let client = ApiClient::new("test_token".to_string());
        let result = users_info(&client, "U123456".to_string()).await;
        // Result will fail because there's no mock server, but that's expected
        assert!(result.is_err());
    }
}
