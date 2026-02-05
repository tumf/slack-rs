//! Filtering functionality for conversations

use crate::api::ApiResponse;
use serde_json::Value;
use thiserror::Error;

/// Filter error types
#[derive(Debug, Error)]
pub enum FilterError {
    #[error("Invalid filter format: {0}")]
    InvalidFormat(String),
    #[error("Invalid boolean value: {0}")]
    InvalidBoolean(String),
}

/// Filter type for conversation list
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationFilter {
    /// Filter by name pattern (glob)
    Name(String),
    /// Filter by membership status
    IsMember(bool),
    /// Filter by private/public status
    IsPrivate(bool),
}

impl ConversationFilter {
    /// Parse filter from string format "key:value"
    pub fn parse(s: &str) -> Result<Self, FilterError> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(FilterError::InvalidFormat(format!(
                "Expected format 'key:value', got '{}'",
                s
            )));
        }

        match parts[0] {
            "name" => Ok(ConversationFilter::Name(parts[1].to_string())),
            "is_member" => {
                let value = parts[1].parse::<bool>().map_err(|_| {
                    FilterError::InvalidBoolean(format!(
                        "Expected 'true' or 'false', got '{}'",
                        parts[1]
                    ))
                })?;
                Ok(ConversationFilter::IsMember(value))
            }
            "is_private" => {
                let value = parts[1].parse::<bool>().map_err(|_| {
                    FilterError::InvalidBoolean(format!(
                        "Expected 'true' or 'false', got '{}'",
                        parts[1]
                    ))
                })?;
                Ok(ConversationFilter::IsPrivate(value))
            }
            _ => Err(FilterError::InvalidFormat(format!(
                "Unknown filter key: {}",
                parts[0]
            ))),
        }
    }

    /// Apply filter to conversation JSON value
    pub fn matches(&self, conv: &Value) -> bool {
        match self {
            ConversationFilter::Name(pattern) => {
                if let Some(name) = conv.get("name").and_then(|v| v.as_str()) {
                    glob_match(pattern, name)
                } else {
                    false
                }
            }
            ConversationFilter::IsMember(expected) => {
                if let Some(is_member) = conv.get("is_member").and_then(|v| v.as_bool()) {
                    is_member == *expected
                } else {
                    false
                }
            }
            ConversationFilter::IsPrivate(expected) => {
                if let Some(is_private) = conv.get("is_private").and_then(|v| v.as_bool()) {
                    is_private == *expected
                } else {
                    false
                }
            }
        }
    }
}

/// Simple glob pattern matching (supports * wildcard)
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let pattern_parts: Vec<&str> = pattern.split('*').collect();

    // No wildcard - exact match
    if pattern_parts.len() == 1 {
        return pattern == text;
    }

    // Pattern starts with wildcard
    if pattern.starts_with('*') && pattern_parts.len() == 2 && pattern_parts[1].is_empty() {
        return true; // Pattern is just "*"
    }

    // Pattern ends with wildcard
    if pattern.ends_with('*') && pattern_parts.len() == 2 && pattern_parts[0].is_empty() {
        return true; // Pattern is just "*"
    }

    // General wildcard matching
    let mut text_pos = 0;

    for (i, part) in pattern_parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 && !pattern.starts_with('*') {
            // First part must match at the beginning
            if !text[text_pos..].starts_with(part) {
                return false;
            }
            text_pos += part.len();
        } else if i == pattern_parts.len() - 1 && !pattern.ends_with('*') {
            // Last part must match at the end
            if !text.ends_with(part) {
                return false;
            }
        } else {
            // Middle part - find it
            if let Some(pos) = text[text_pos..].find(part) {
                text_pos += pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

/// Apply multiple filters with AND logic to conversations
pub fn apply_filters(response: &mut ApiResponse, filters: &[ConversationFilter]) {
    if filters.is_empty() {
        return;
    }

    if let Some(channels) = response.data.get_mut("channels") {
        if let Some(channels_array) = channels.as_array_mut() {
            channels_array.retain(|conv| filters.iter().all(|filter| filter.matches(conv)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_filter_parse_name() {
        let filter = ConversationFilter::parse("name:test*").unwrap();
        assert_eq!(filter, ConversationFilter::Name("test*".to_string()));
    }

    #[test]
    fn test_filter_parse_is_member() {
        let filter = ConversationFilter::parse("is_member:true").unwrap();
        assert_eq!(filter, ConversationFilter::IsMember(true));

        let filter = ConversationFilter::parse("is_member:false").unwrap();
        assert_eq!(filter, ConversationFilter::IsMember(false));
    }

    #[test]
    fn test_filter_parse_is_private() {
        let filter = ConversationFilter::parse("is_private:true").unwrap();
        assert_eq!(filter, ConversationFilter::IsPrivate(true));

        let filter = ConversationFilter::parse("is_private:false").unwrap();
        assert_eq!(filter, ConversationFilter::IsPrivate(false));
    }

    #[test]
    fn test_filter_parse_invalid_format() {
        let result = ConversationFilter::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_parse_invalid_key() {
        let result = ConversationFilter::parse("unknown:value");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_parse_invalid_boolean() {
        let result = ConversationFilter::parse("is_member:maybe");
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("test", "test"));
        assert!(!glob_match("test", "other"));
    }

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("test*", "test"));
        assert!(glob_match("test*", "test123"));
        assert!(!glob_match("test*", "other"));

        assert!(glob_match("*test", "test"));
        assert!(glob_match("*test", "mytest"));
        assert!(!glob_match("*test", "testing"));

        assert!(glob_match("*test*", "test"));
        assert!(glob_match("*test*", "mytest123"));
        assert!(!glob_match("*test*", "other"));
    }

    #[test]
    fn test_filter_matches_name() {
        let filter = ConversationFilter::Name("general".to_string());
        let conv = json!({"name": "general", "id": "C123"});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "random", "id": "C124"});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_filter_matches_name_glob() {
        let filter = ConversationFilter::Name("test*".to_string());
        let conv = json!({"name": "test-channel", "id": "C123"});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "other", "id": "C124"});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_filter_matches_is_member() {
        let filter = ConversationFilter::IsMember(true);
        let conv = json!({"name": "general", "is_member": true});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "general", "is_member": false});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_filter_matches_is_private() {
        let filter = ConversationFilter::IsPrivate(true);
        let conv = json!({"name": "private", "is_private": true});
        assert!(filter.matches(&conv));

        let conv = json!({"name": "public", "is_private": false});
        assert!(!filter.matches(&conv));
    }

    #[test]
    fn test_apply_filters_and_condition() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "test-public", "is_member": true, "is_private": false},
                    {"id": "C2", "name": "test-private", "is_member": true, "is_private": true},
                    {"id": "C3", "name": "other", "is_member": true, "is_private": false},
                    {"id": "C4", "name": "test-nomember", "is_member": false, "is_private": false},
                ]),
            )]),
            error: None,
        };

        let filters = vec![
            ConversationFilter::Name("test*".to_string()),
            ConversationFilter::IsMember(true),
        ];

        apply_filters(&mut response, &filters);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels.len(), 2); // C1 and C2
        assert_eq!(channels[0].get("id").unwrap().as_str().unwrap(), "C1");
        assert_eq!(channels[1].get("id").unwrap().as_str().unwrap(), "C2");
    }
}
