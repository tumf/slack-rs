//! Error guidance for Slack API errors
//!
//! This module provides user-friendly guidance for common Slack API errors,
//! helping users understand the cause and how to resolve issues.

use std::collections::HashMap;

/// Error guidance information
#[derive(Debug, Clone)]
pub struct ErrorGuidance {
    /// Error code from Slack API
    pub error_code: String,
    /// Human-readable cause description
    pub cause: String,
    /// Suggested resolution steps
    pub resolution: String,
}

impl ErrorGuidance {
    /// Create a new error guidance entry
    pub fn new(error_code: &str, cause: &str, resolution: &str) -> Self {
        Self {
            error_code: error_code.to_string(),
            cause: cause.to_string(),
            resolution: resolution.to_string(),
        }
    }
}

/// Get error guidance for a given error code
pub fn get_error_guidance(error_code: &str) -> Option<ErrorGuidance> {
    let guidance_map = build_guidance_map();
    guidance_map.get(error_code).cloned()
}

/// Build the complete error guidance mapping
fn build_guidance_map() -> HashMap<String, ErrorGuidance> {
    let mut map = HashMap::new();

    // not_allowed_token_type
    map.insert(
        "not_allowed_token_type".to_string(),
        ErrorGuidance::new(
            "not_allowed_token_type",
            "The token type used for this request is not allowed for this API method",
            "Use a different token type (bot or user). Try: --token-type user or --token-type bot",
        ),
    );

    // missing_scope
    map.insert(
        "missing_scope".to_string(),
        ErrorGuidance::new(
            "missing_scope",
            "The token does not have the required OAuth scope for this API method",
            "Re-authenticate with the required scopes. Run: slack auth login",
        ),
    );

    // invalid_auth
    map.insert(
        "invalid_auth".to_string(),
        ErrorGuidance::new(
            "invalid_auth",
            "The authentication token is invalid, expired, or revoked",
            "Re-authenticate to obtain a new token. Run: slack auth login",
        ),
    );

    // token_revoked
    map.insert(
        "token_revoked".to_string(),
        ErrorGuidance::new(
            "token_revoked",
            "The authentication token has been revoked",
            "Re-authenticate to obtain a new token. Run: slack auth login",
        ),
    );

    // token_expired
    map.insert(
        "token_expired".to_string(),
        ErrorGuidance::new(
            "token_expired",
            "The authentication token has expired",
            "Re-authenticate to obtain a new token. Run: slack auth login",
        ),
    );

    // not_authed
    map.insert(
        "not_authed".to_string(),
        ErrorGuidance::new(
            "not_authed",
            "No authentication token was provided",
            "Authenticate first. Run: slack auth login",
        ),
    );

    // account_inactive
    map.insert(
        "account_inactive".to_string(),
        ErrorGuidance::new(
            "account_inactive",
            "The authentication token is for a deleted user or workspace",
            "Use a valid workspace account and re-authenticate. Run: slack auth login",
        ),
    );

    // no_permission
    map.insert(
        "no_permission".to_string(),
        ErrorGuidance::new(
            "no_permission",
            "The token does not have permission to perform this action",
            "Check workspace permissions or use a token with appropriate privileges",
        ),
    );

    // org_login_required
    map.insert(
        "org_login_required".to_string(),
        ErrorGuidance::new(
            "org_login_required",
            "The workspace requires organization-wide login",
            "Contact your workspace administrator for access",
        ),
    );

    // ekm_access_denied
    map.insert(
        "ekm_access_denied".to_string(),
        ErrorGuidance::new(
            "ekm_access_denied",
            "Enterprise Key Management (EKM) access was denied",
            "Contact your workspace administrator to check EKM settings",
        ),
    );

    map
}

/// Format error guidance for display on stderr
pub fn format_error_guidance(error_code: &str) -> Option<String> {
    get_error_guidance(error_code).map(|guidance| {
        format!(
            "\nError: {}\nCause: {}\nResolution: {}\n",
            guidance.error_code, guidance.cause, guidance.resolution
        )
    })
}

/// Display error guidance for wrapper command responses (ApiResponse)
///
/// Checks if the ApiResponse contains an error and displays guidance to stderr if available.
/// This function is designed for wrapper commands (msg, react, search, conv, users, file).
///
/// # Arguments
/// * `response` - The API response from a wrapper command
pub fn display_wrapper_error_guidance(response: &crate::api::types::ApiResponse) {
    // Check if response indicates an error
    if !response.ok {
        // Try to get error code from response
        if let Some(error_code) = &response.error {
            // Display guidance if available
            if let Some(guidance) = format_error_guidance(error_code) {
                eprintln!("{}", guidance);
            }
        }
    }
}

/// Display error guidance for JSON value responses (for commands like file upload)
///
/// Checks if the JSON value contains an error and displays guidance to stderr if available.
///
/// # Arguments
/// * `response` - The response as a JSON value
pub fn display_json_error_guidance(response: &serde_json::Value) {
    // Check if response indicates an error
    if let Some(ok) = response.get("ok").and_then(|v| v.as_bool()) {
        if !ok {
            // Try to get error code from response
            if let Some(error_code) = response.get("error").and_then(|v| v.as_str()) {
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

    #[test]
    fn test_get_error_guidance_not_allowed_token_type() {
        let guidance = get_error_guidance("not_allowed_token_type");
        assert!(guidance.is_some());
        let guidance = guidance.unwrap();
        assert_eq!(guidance.error_code, "not_allowed_token_type");
        assert!(guidance.cause.contains("token type"));
        assert!(guidance.resolution.contains("token-type"));
    }

    #[test]
    fn test_get_error_guidance_missing_scope() {
        let guidance = get_error_guidance("missing_scope");
        assert!(guidance.is_some());
        let guidance = guidance.unwrap();
        assert_eq!(guidance.error_code, "missing_scope");
        assert!(guidance.cause.contains("scope"));
        assert!(guidance.resolution.contains("login"));
    }

    #[test]
    fn test_get_error_guidance_invalid_auth() {
        let guidance = get_error_guidance("invalid_auth");
        assert!(guidance.is_some());
        let guidance = guidance.unwrap();
        assert_eq!(guidance.error_code, "invalid_auth");
        assert!(guidance.cause.contains("invalid"));
        assert!(guidance.resolution.contains("login"));
    }

    #[test]
    fn test_get_error_guidance_unknown_error() {
        let guidance = get_error_guidance("unknown_error_code");
        assert!(guidance.is_none());
    }

    #[test]
    fn test_format_error_guidance() {
        let formatted = format_error_guidance("missing_scope");
        assert!(formatted.is_some());
        let formatted = formatted.unwrap();
        assert!(formatted.contains("Error: missing_scope"));
        assert!(formatted.contains("Cause:"));
        assert!(formatted.contains("Resolution:"));
    }

    #[test]
    fn test_format_error_guidance_unknown() {
        let formatted = format_error_guidance("unknown_error");
        assert!(formatted.is_none());
    }

    #[test]
    fn test_display_wrapper_error_guidance_with_known_error() {
        use crate::api::types::ApiResponse;
        use std::collections::HashMap;

        // Create response with known error code
        let response = ApiResponse {
            ok: false,
            data: HashMap::new(),
            error: Some("missing_scope".to_string()),
        };

        // This should not panic - guidance should be displayed to stderr
        display_wrapper_error_guidance(&response);
    }

    #[test]
    fn test_display_wrapper_error_guidance_with_unknown_error() {
        use crate::api::types::ApiResponse;
        use std::collections::HashMap;

        // Create response with unknown error code
        let response = ApiResponse {
            ok: false,
            data: HashMap::new(),
            error: Some("unknown_error_code".to_string()),
        };

        // This should not panic - no guidance for unknown errors
        display_wrapper_error_guidance(&response);
    }

    #[test]
    fn test_display_wrapper_error_guidance_with_success() {
        use crate::api::types::ApiResponse;
        use std::collections::HashMap;

        // Create successful response
        let mut data = HashMap::new();
        data.insert("channel".to_string(), serde_json::json!("C123456"));

        let response = ApiResponse {
            ok: true,
            data,
            error: None,
        };

        // This should not display anything (success case)
        display_wrapper_error_guidance(&response);
    }

    #[test]
    fn test_display_wrapper_error_guidance_with_not_allowed_token_type() {
        use crate::api::types::ApiResponse;
        use std::collections::HashMap;

        // Create response with not_allowed_token_type error
        let response = ApiResponse {
            ok: false,
            data: HashMap::new(),
            error: Some("not_allowed_token_type".to_string()),
        };

        // This should display guidance to stderr
        display_wrapper_error_guidance(&response);
    }

    #[test]
    fn test_display_wrapper_error_guidance_with_invalid_auth() {
        use crate::api::types::ApiResponse;
        use std::collections::HashMap;

        // Create response with invalid_auth error
        let response = ApiResponse {
            ok: false,
            data: HashMap::new(),
            error: Some("invalid_auth".to_string()),
        };

        // This should display guidance to stderr
        display_wrapper_error_guidance(&response);
    }

    #[test]
    fn test_display_json_error_guidance_with_known_error() {
        use serde_json::json;

        // Create JSON response with known error code
        let response = json!({
            "ok": false,
            "error": "missing_scope"
        });

        // This should not panic - guidance should be displayed to stderr
        display_json_error_guidance(&response);
    }

    #[test]
    fn test_display_json_error_guidance_with_unknown_error() {
        use serde_json::json;

        // Create JSON response with unknown error code
        let response = json!({
            "ok": false,
            "error": "unknown_error_code"
        });

        // This should not panic - no guidance for unknown errors
        display_json_error_guidance(&response);
    }

    #[test]
    fn test_display_json_error_guidance_with_success() {
        use serde_json::json;

        // Create successful JSON response
        let response = json!({
            "ok": true,
            "file": {
                "id": "F123456"
            }
        });

        // This should not display anything (success case)
        display_json_error_guidance(&response);
    }

    #[test]
    fn test_display_json_error_guidance_with_not_allowed_token_type() {
        use serde_json::json;

        // Create JSON response with not_allowed_token_type error
        let response = json!({
            "ok": false,
            "error": "not_allowed_token_type"
        });

        // This should display guidance to stderr
        display_json_error_guidance(&response);
    }
}
