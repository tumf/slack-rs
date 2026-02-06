//! Argument parsing for `api call` command
//!
//! Parses command-line arguments into structured API call parameters:
//! - Method name (e.g., "chat.postMessage")
//! - Key-value pairs (e.g., "channel=C123456" "text=hello")
//! - Flags: --json, --get

use crate::profile::TokenType;
use serde_json::{json, Value};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgsError {
    #[error("Missing method argument")]
    MissingMethod,

    #[error("Invalid key-value pair: {0}")]
    InvalidKeyValue(String),

    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
}

pub type Result<T> = std::result::Result<T, ArgsError>;

/// Parsed API call arguments
#[derive(Debug, Clone, PartialEq)]
pub struct ApiCallArgs {
    /// API method name (e.g., "chat.postMessage")
    pub method: String,

    /// Request parameters
    pub params: HashMap<String, String>,

    /// Use JSON body instead of form encoding
    pub use_json: bool,

    /// Use GET method instead of POST
    pub use_get: bool,

    /// Token type preference (CLI flag override)
    pub token_type: Option<TokenType>,

    /// Output raw Slack API response without envelope
    pub raw: bool,
}

impl ApiCallArgs {
    /// Parse arguments from command-line args
    pub fn parse(args: &[String]) -> Result<Self> {
        if args.is_empty() {
            return Err(ArgsError::MissingMethod);
        }

        let method = args[0].clone();
        let mut params = HashMap::new();
        let mut use_json = false;
        let mut use_get = false;
        let mut token_type = None;

        // Check SLACKRS_OUTPUT environment variable for default output mode
        // --raw flag will override this
        let mut raw = if let Ok(output_mode) = std::env::var("SLACKRS_OUTPUT") {
            output_mode.trim().to_lowercase() == "raw"
        } else {
            false
        };

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--json" {
                use_json = true;
            } else if arg == "--get" {
                use_get = true;
            } else if arg == "--raw" {
                // --raw flag always overrides environment variable
                raw = true;
            } else if arg == "--token-type" {
                // Space-separated format: --token-type VALUE
                i += 1;
                if i < args.len() {
                    token_type = Some(
                        args[i]
                            .parse::<TokenType>()
                            .map_err(|e| ArgsError::InvalidJson(e.to_string()))?,
                    );
                }
            } else if arg.starts_with("--token-type=") {
                // Equals format: --token-type=VALUE
                if let Some(value) = arg.strip_prefix("--token-type=") {
                    token_type = Some(
                        value
                            .parse::<TokenType>()
                            .map_err(|e| ArgsError::InvalidJson(e.to_string()))?,
                    );
                }
            } else if arg.starts_with("--") {
                // Ignore unknown flags for forward compatibility
            } else {
                // Parse key=value
                if let Some((key, value)) = arg.split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                } else {
                    return Err(ArgsError::InvalidKeyValue(arg.clone()));
                }
            }
            i += 1;
        }

        Ok(Self {
            method,
            params,
            use_json,
            use_get,
            token_type,
            raw,
        })
    }

    /// Convert to JSON body
    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (k, v) in &self.params {
            map.insert(k.clone(), Value::String(v.clone()));
        }
        Value::Object(map)
    }

    /// Convert to form parameters
    pub fn to_form(&self) -> Vec<(String, String)> {
        self.params
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let args = vec!["chat.postMessage".to_string()];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert_eq!(result.method, "chat.postMessage");
        assert!(result.params.is_empty());
        assert!(!result.use_json);
        assert!(!result.use_get);
        assert_eq!(result.token_type, None);
    }

    #[test]
    fn test_parse_with_params() {
        let args = vec![
            "chat.postMessage".to_string(),
            "channel=C123456".to_string(),
            "text=Hello World".to_string(),
        ];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert_eq!(result.method, "chat.postMessage");
        assert_eq!(result.params.get("channel"), Some(&"C123456".to_string()));
        assert_eq!(result.params.get("text"), Some(&"Hello World".to_string()));
    }

    #[test]
    fn test_parse_with_json_flag() {
        let args = vec![
            "chat.postMessage".to_string(),
            "--json".to_string(),
            "channel=C123456".to_string(),
        ];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert_eq!(result.method, "chat.postMessage");
        assert!(result.use_json);
        assert!(!result.use_get);
    }

    #[test]
    fn test_parse_with_get_flag() {
        let args = vec![
            "users.info".to_string(),
            "--get".to_string(),
            "user=U123456".to_string(),
        ];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert_eq!(result.method, "users.info");
        assert!(!result.use_json);
        assert!(result.use_get);
    }

    #[test]
    fn test_parse_with_both_flags() {
        let args = vec![
            "chat.postMessage".to_string(),
            "--json".to_string(),
            "--get".to_string(),
            "channel=C123456".to_string(),
        ];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert!(result.use_json);
        assert!(result.use_get);
    }

    #[test]
    fn test_parse_missing_method() {
        let args: Vec<String> = vec![];
        let result = ApiCallArgs::parse(&args);

        assert!(result.is_err());
        match result {
            Err(ArgsError::MissingMethod) => {}
            _ => panic!("Expected MissingMethod error"),
        }
    }

    #[test]
    fn test_parse_invalid_key_value() {
        let args = vec!["chat.postMessage".to_string(), "invalid_arg".to_string()];
        let result = ApiCallArgs::parse(&args);

        assert!(result.is_err());
        match result {
            Err(ArgsError::InvalidKeyValue(arg)) => {
                assert_eq!(arg, "invalid_arg");
            }
            _ => panic!("Expected InvalidKeyValue error"),
        }
    }

    #[test]
    fn test_to_json() {
        let args = ApiCallArgs {
            method: "chat.postMessage".to_string(),
            params: [
                ("channel".to_string(), "C123456".to_string()),
                ("text".to_string(), "Hello".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            use_json: true,
            use_get: false,
            token_type: None,
            raw: false,
        };

        let json = args.to_json();
        assert_eq!(json["channel"], "C123456");
        assert_eq!(json["text"], "Hello");
    }

    #[test]
    fn test_to_form() {
        let args = ApiCallArgs {
            method: "chat.postMessage".to_string(),
            params: [
                ("channel".to_string(), "C123456".to_string()),
                ("text".to_string(), "Hello".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            use_json: false,
            use_get: false,
            token_type: None,
            raw: false,
        };

        let form = args.to_form();
        assert_eq!(form.len(), 2);
        assert!(form.contains(&("channel".to_string(), "C123456".to_string())));
        assert!(form.contains(&("text".to_string(), "Hello".to_string())));
    }

    #[test]
    fn test_parse_token_type_space_separated() {
        let args = vec![
            "chat.postMessage".to_string(),
            "--token-type".to_string(),
            "user".to_string(),
            "channel=C123456".to_string(),
        ];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert_eq!(result.method, "chat.postMessage");
        assert_eq!(result.token_type, Some(TokenType::User));
    }

    #[test]
    fn test_parse_token_type_equals_format() {
        let args = vec![
            "chat.postMessage".to_string(),
            "--token-type=bot".to_string(),
            "channel=C123456".to_string(),
        ];
        let result = ApiCallArgs::parse(&args).unwrap();

        assert_eq!(result.method, "chat.postMessage");
        assert_eq!(result.token_type, Some(TokenType::Bot));
    }

    #[test]
    fn test_parse_token_type_both_formats() {
        // Test space-separated with bot
        let args1 = vec![
            "users.info".to_string(),
            "--token-type".to_string(),
            "bot".to_string(),
        ];
        let result1 = ApiCallArgs::parse(&args1).unwrap();
        assert_eq!(result1.token_type, Some(TokenType::Bot));

        // Test equals format with user
        let args2 = vec!["users.info".to_string(), "--token-type=user".to_string()];
        let result2 = ApiCallArgs::parse(&args2).unwrap();
        assert_eq!(result2.token_type, Some(TokenType::User));
    }
}
