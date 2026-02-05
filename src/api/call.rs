//! API call handler with metadata attachment
//!
//! Executes API calls and enriches responses with execution context:
//! - Profile name
//! - Team ID
//! - User ID
//! - Method name

use super::args::ApiCallArgs;
use super::client::{ApiClient, RequestBody};
use super::guidance::format_error_guidance;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiCallError {
    #[error("Client error: {0}")]
    ClientError(#[from] super::client::ApiClientError),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, ApiCallError>;

/// Execution context for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallContext {
    pub profile_name: Option<String>,
    pub team_id: String,
    pub user_id: String,
}

/// API call response with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiCallResponse {
    /// Original API response
    pub response: Value,

    /// Execution metadata
    pub meta: ApiCallMeta,
}

/// Execution metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiCallMeta {
    pub profile_name: Option<String>,
    pub team_id: String,
    pub user_id: String,
    pub method: String,
    pub command: String,
    pub token_type: String,
}

/// Execute an API call with the given arguments, context, token type, and command name
pub async fn execute_api_call(
    client: &ApiClient,
    args: &ApiCallArgs,
    token: &str,
    context: &ApiCallContext,
    token_type: &str,
    command: &str,
) -> Result<ApiCallResponse> {
    // Determine HTTP method
    let method = if args.use_get {
        Method::GET
    } else {
        Method::POST
    };

    // Prepare request body
    let body = if args.use_json {
        RequestBody::Json(args.to_json())
    } else if method == Method::POST {
        RequestBody::Form(args.to_form())
    } else {
        RequestBody::None
    };

    // Make the API call
    let response = client.call(method, &args.method, token, body).await?;

    // Parse response body
    let response_text = response
        .text()
        .await
        .map_err(|e| ApiCallError::ParseError(e.to_string()))?;

    let response_json: Value = serde_json::from_str(&response_text)
        .map_err(|e| ApiCallError::ParseError(e.to_string()))?;

    // Construct response with metadata
    let api_response = ApiCallResponse {
        response: response_json,
        meta: ApiCallMeta {
            profile_name: context.profile_name.clone(),
            team_id: context.team_id.clone(),
            user_id: context.user_id.clone(),
            method: args.method.clone(),
            command: command.to_string(),
            token_type: token_type.to_string(),
        },
    };

    Ok(api_response)
}

/// Display error guidance to stderr if the response contains a known error
pub fn display_error_guidance(response: &ApiCallResponse) {
    // Check if response has an error
    if let Some(ok) = response.response.get("ok").and_then(|v| v.as_bool()) {
        if !ok {
            // Try to get error code from response
            if let Some(error_code) = response.response.get("error").and_then(|v| v.as_str()) {
                // Display guidance if available
                if let Some(guidance) = format_error_guidance(error_code) {
                    eprintln!("{}", guidance);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_api_call_meta_serialization() {
        let meta = ApiCallMeta {
            profile_name: Some("default".to_string()),
            team_id: "T123ABC".to_string(),
            user_id: "U456DEF".to_string(),
            method: "chat.postMessage".to_string(),
            command: "api call".to_string(),
            token_type: "bot".to_string(),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: ApiCallMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.profile_name, Some("default".to_string()));
        assert_eq!(deserialized.team_id, "T123ABC");
        assert_eq!(deserialized.user_id, "U456DEF");
        assert_eq!(deserialized.method, "chat.postMessage");
        assert_eq!(deserialized.command, "api call");
        assert_eq!(deserialized.token_type, "bot");
    }

    #[test]
    fn test_api_call_response_structure() {
        let response = ApiCallResponse {
            response: json!({
                "ok": true,
                "channel": "C123456",
                "ts": "1234567890.123456"
            }),
            meta: ApiCallMeta {
                profile_name: Some("work".to_string()),
                team_id: "T123ABC".to_string(),
                user_id: "U456DEF".to_string(),
                method: "chat.postMessage".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        let json = serde_json::to_value(&response).unwrap();

        assert!(json["response"]["ok"].as_bool().unwrap());
        assert_eq!(json["meta"]["team_id"], "T123ABC");
        assert_eq!(json["meta"]["method"], "chat.postMessage");
        assert_eq!(json["meta"]["command"], "api call");
        assert_eq!(json["meta"]["token_type"], "bot");
    }

    #[test]
    fn test_display_error_guidance_with_known_error() {
        // Create response with known error code
        let response = ApiCallResponse {
            response: json!({
                "ok": false,
                "error": "missing_scope"
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123ABC".to_string(),
                user_id: "U456DEF".to_string(),
                method: "chat.postMessage".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // This should not panic - guidance should be displayed to stderr
        display_error_guidance(&response);
    }

    #[test]
    fn test_display_error_guidance_with_unknown_error() {
        // Create response with unknown error code
        let response = ApiCallResponse {
            response: json!({
                "ok": false,
                "error": "unknown_error_code"
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123ABC".to_string(),
                user_id: "U456DEF".to_string(),
                method: "chat.postMessage".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // This should not panic - no guidance for unknown errors
        display_error_guidance(&response);
    }

    #[test]
    fn test_display_error_guidance_with_success() {
        // Create successful response
        let response = ApiCallResponse {
            response: json!({
                "ok": true,
                "channel": "C123456"
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123ABC".to_string(),
                user_id: "U456DEF".to_string(),
                method: "chat.postMessage".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // This should not display anything (success case)
        display_error_guidance(&response);
    }

    #[test]
    fn test_display_error_guidance_with_not_allowed_token_type() {
        // Create response with not_allowed_token_type error
        let response = ApiCallResponse {
            response: json!({
                "ok": false,
                "error": "not_allowed_token_type"
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123ABC".to_string(),
                user_id: "U456DEF".to_string(),
                method: "conversations.history".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // This should display guidance to stderr
        display_error_guidance(&response);
    }

    #[test]
    fn test_display_error_guidance_with_invalid_auth() {
        // Create response with invalid_auth error
        let response = ApiCallResponse {
            response: json!({
                "ok": false,
                "error": "invalid_auth"
            }),
            meta: ApiCallMeta {
                profile_name: Some("default".to_string()),
                team_id: "T123ABC".to_string(),
                user_id: "U456DEF".to_string(),
                method: "auth.test".to_string(),
                command: "api call".to_string(),
                token_type: "bot".to_string(),
            },
        };

        // This should display guidance to stderr
        display_error_guidance(&response);
    }
}
