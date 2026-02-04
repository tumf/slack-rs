//! OAuth scope preset definitions and expansion utilities
//!
//! This module provides preset scope definitions (like "all") and utilities
//! to expand them into concrete scope lists.

use std::collections::BTreeSet;

/// Returns the comprehensive bot scopes preset
///
/// This includes common bot scopes but excludes admin/Enterprise-only scopes.
/// The list is stable and sorted alphabetically.
pub fn bot_all_scopes() -> Vec<String> {
    vec![
        "channels:history",
        "channels:read",
        "channels:write",
        "chat:write",
        "conversations.connect:read",
        "conversations.connect:write",
        "dnd:read",
        "dnd:write",
        "emoji:read",
        "files:read",
        "files:write",
        "groups:history",
        "groups:read",
        "groups:write",
        "im:history",
        "im:read",
        "im:write",
        "mpim:history",
        "mpim:read",
        "mpim:write",
        "pins:read",
        "pins:write",
        "reactions:read",
        "reactions:write",
        "reminders:read",
        "reminders:write",
        "team:read",
        "usergroups:read",
        "usergroups:write",
        "users.profile:read",
        "users.profile:write",
        "users:read",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Returns the comprehensive user scopes preset
///
/// This includes common user scopes but excludes admin/Enterprise-only scopes.
/// The list is stable and sorted alphabetically.
pub fn user_all_scopes() -> Vec<String> {
    vec![
        "channels:history",
        "channels:read",
        "channels:write",
        "chat:write",
        "dnd:read",
        "dnd:write",
        "emoji:read",
        "files:read",
        "files:write",
        "groups:history",
        "groups:read",
        "groups:write",
        "im:history",
        "im:read",
        "im:write",
        "mpim:history",
        "mpim:read",
        "mpim:write",
        "pins:read",
        "pins:write",
        "reactions:read",
        "reactions:write",
        "reminders:read",
        "reminders:write",
        "search:read",
        "stars:read",
        "stars:write",
        "team:read",
        "usergroups:read",
        "usergroups:write",
        "users.profile:read",
        "users.profile:write",
        "users:read",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Returns the comprehensive "all" scope preset (legacy, defaults to bot scopes)
///
/// This includes common scopes for bot/user tokens but excludes admin/Enterprise-only scopes.
/// The list is stable and sorted alphabetically.
/// For new code, prefer bot_all_scopes() or user_all_scopes() explicitly.
pub fn all_scopes() -> Vec<String> {
    bot_all_scopes()
}

/// Returns bot-specific scopes
///
/// This includes scopes typically used for bot tokens.
#[allow(dead_code)]
pub fn bot_scopes() -> Vec<String> {
    vec![
        "channels:history",
        "channels:read",
        "channels:write",
        "chat:write",
        "conversations.connect:read",
        "conversations.connect:write",
        "emoji:read",
        "files:read",
        "files:write",
        "groups:history",
        "groups:read",
        "groups:write",
        "im:history",
        "im:read",
        "im:write",
        "mpim:history",
        "mpim:read",
        "mpim:write",
        "pins:read",
        "pins:write",
        "reactions:read",
        "reactions:write",
        "reminders:read",
        "reminders:write",
        "usergroups:read",
        "usergroups:write",
        "users:read",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Returns user-specific scopes
///
/// This includes scopes typically used for user tokens.
#[allow(dead_code)]
pub fn user_scopes() -> Vec<String> {
    vec![
        "channels:history",
        "channels:read",
        "channels:write",
        "chat:write",
        "dnd:read",
        "dnd:write",
        "files:read",
        "files:write",
        "groups:history",
        "groups:read",
        "groups:write",
        "im:history",
        "im:read",
        "im:write",
        "mpim:history",
        "mpim:read",
        "mpim:write",
        "pins:read",
        "pins:write",
        "reactions:read",
        "reactions:write",
        "reminders:read",
        "reminders:write",
        "search:read",
        "stars:read",
        "stars:write",
        "team:read",
        "users.profile:read",
        "users.profile:write",
        "users:read",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Expands preset names (like "all", "bot:all", "user:all") in a scope list and removes duplicates
///
/// # Arguments
/// * `input_scopes` - List of scopes which may include preset names (case-insensitive)
///
/// # Returns
/// A deduplicated, sorted list of concrete scopes with presets expanded
///
/// # Presets
/// - "all": Expands to bot_all_scopes() (legacy behavior, for backward compatibility)
/// - "bot:all": Expands to bot_all_scopes()
/// - "user:all": Expands to user_all_scopes()
///
/// # Example
/// ```
/// use slack_rs::oauth::scopes::expand_scopes;
///
/// let result = expand_scopes(&["bot:all".to_string(), "custom:scope".to_string()]);
/// assert!(result.contains(&"chat:write".to_string()));
/// assert!(result.contains(&"custom:scope".to_string()));
/// ```
pub fn expand_scopes(input_scopes: &[String]) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for scope in input_scopes {
        let normalized = scope.trim().to_lowercase();

        match normalized.as_str() {
            "all" => {
                // Legacy: expand to bot scopes for backward compatibility
                for preset_scope in all_scopes() {
                    expanded.insert(preset_scope);
                }
            }
            "bot:all" => {
                // Explicit bot scopes preset
                for preset_scope in bot_all_scopes() {
                    expanded.insert(preset_scope);
                }
            }
            "user:all" => {
                // Explicit user scopes preset
                for preset_scope in user_all_scopes() {
                    expanded.insert(preset_scope);
                }
            }
            _ => {
                // Keep individual scopes as-is (preserving original case)
                expanded.insert(scope.trim().to_string());
            }
        }
    }

    expanded.into_iter().collect()
}

/// Expands scopes with context-aware "all" preset
///
/// # Arguments
/// * `input_scopes` - List of scopes which may include preset names
/// * `is_bot_context` - true for bot scope context, false for user scope context
///
/// # Returns
/// A deduplicated, sorted list of concrete scopes with presets expanded
///
/// # Context-aware behavior
/// - In bot context, "all" expands to bot_all_scopes()
/// - In user context, "all" expands to user_all_scopes()
/// - "bot:all" and "user:all" always expand to their respective presets regardless of context
pub fn expand_scopes_with_context(input_scopes: &[String], is_bot_context: bool) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for scope in input_scopes {
        let normalized = scope.trim().to_lowercase();

        match normalized.as_str() {
            "all" => {
                // Context-aware expansion
                let preset = if is_bot_context {
                    bot_all_scopes()
                } else {
                    user_all_scopes()
                };
                for preset_scope in preset {
                    expanded.insert(preset_scope);
                }
            }
            "bot:all" => {
                for preset_scope in bot_all_scopes() {
                    expanded.insert(preset_scope);
                }
            }
            "user:all" => {
                for preset_scope in user_all_scopes() {
                    expanded.insert(preset_scope);
                }
            }
            _ => {
                expanded.insert(scope.trim().to_string());
            }
        }
    }

    expanded.into_iter().collect()
}

/// Expands preset names for bot scopes with context awareness
///
/// # Arguments
/// * `input_scopes` - List of scopes which may include preset names like "all", "bot:all"
///
/// # Returns
/// A deduplicated, sorted list of concrete scopes with presets expanded
///
/// "all" is interpreted as "bot:all" in bot context
#[allow(dead_code)]
pub fn expand_bot_scopes(input_scopes: &[String]) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for scope in input_scopes {
        let normalized = scope.trim().to_lowercase();

        if normalized == "all" || normalized == "bot:all" {
            // Expand the bot preset
            for preset_scope in bot_scopes() {
                expanded.insert(preset_scope);
            }
        } else if normalized == "user:all" {
            // User scopes should not be in bot scopes, but handle gracefully
            for preset_scope in user_scopes() {
                expanded.insert(preset_scope);
            }
        } else {
            // Keep individual scopes as-is (preserving original case)
            expanded.insert(scope.trim().to_string());
        }
    }

    expanded.into_iter().collect()
}

/// Expands preset names for user scopes with context awareness
///
/// # Arguments
/// * `input_scopes` - List of scopes which may include preset names like "all", "user:all"
///
/// # Returns
/// A deduplicated, sorted list of concrete scopes with presets expanded
///
/// "all" is interpreted as "user:all" in user context
#[allow(dead_code)]
pub fn expand_user_scopes(input_scopes: &[String]) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for scope in input_scopes {
        let normalized = scope.trim().to_lowercase();

        if normalized == "all" || normalized == "user:all" {
            // Expand the user preset
            for preset_scope in user_scopes() {
                expanded.insert(preset_scope);
            }
        } else if normalized == "bot:all" {
            // Bot scopes should not be in user scopes, but handle gracefully
            for preset_scope in bot_scopes() {
                expanded.insert(preset_scope);
            }
        } else {
            // Keep individual scopes as-is (preserving original case)
            expanded.insert(scope.trim().to_string());
        }
    }

    expanded.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_scopes_returns_stable_list() {
        let scopes1 = all_scopes();
        let scopes2 = all_scopes();
        assert_eq!(scopes1, scopes2, "all_scopes() should return a stable list");
        assert!(!scopes1.is_empty(), "all_scopes() should not be empty");
    }

    #[test]
    fn test_all_scopes_is_sorted() {
        let scopes = all_scopes();
        let mut sorted = scopes.clone();
        sorted.sort();
        assert_eq!(
            scopes, sorted,
            "all_scopes() should be alphabetically sorted"
        );
    }

    #[test]
    fn test_all_scopes_contains_expected_scopes() {
        let scopes = all_scopes();
        assert!(scopes.contains(&"chat:write".to_string()));
        assert!(scopes.contains(&"users:read".to_string()));
        assert!(scopes.contains(&"channels:read".to_string()));
    }

    #[test]
    fn test_expand_scopes_with_all_only() {
        let input = vec!["all".to_string()];
        let result = expand_scopes(&input);

        assert!(result.contains(&"chat:write".to_string()));
        assert!(result.contains(&"users:read".to_string()));
        assert!(result.len() > 10, "all should expand to many scopes");
    }

    #[test]
    fn test_expand_scopes_case_insensitive() {
        let input1 = vec!["all".to_string()];
        let input2 = vec!["ALL".to_string()];
        let input3 = vec!["AlL".to_string()];

        let result1 = expand_scopes(&input1);
        let result2 = expand_scopes(&input2);
        let result3 = expand_scopes(&input3);

        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_expand_scopes_mixed_with_duplicates() {
        let input = vec![
            "all".to_string(),
            "chat:write".to_string(), // duplicate with "all"
            "custom:scope".to_string(),
        ];
        let result = expand_scopes(&input);

        // Should contain all scopes from "all" plus custom:scope
        assert!(result.contains(&"chat:write".to_string()));
        assert!(result.contains(&"users:read".to_string()));
        assert!(result.contains(&"custom:scope".to_string()));

        // Should deduplicate - chat:write appears only once
        assert_eq!(
            result.iter().filter(|s| *s == "chat:write").count(),
            1,
            "chat:write should appear only once"
        );
    }

    #[test]
    fn test_expand_scopes_no_presets() {
        let input = vec!["chat:write".to_string(), "users:read".to_string()];
        let result = expand_scopes(&input);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"chat:write".to_string()));
        assert!(result.contains(&"users:read".to_string()));
    }

    #[test]
    fn test_expand_scopes_empty() {
        let input: Vec<String> = vec![];
        let result = expand_scopes(&input);

        assert!(result.is_empty());
    }

    #[test]
    fn test_expand_scopes_preserves_case_for_non_presets() {
        let input = vec!["Custom:Scope".to_string()];
        let result = expand_scopes(&input);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Custom:Scope");
    }

    #[test]
    fn test_expand_scopes_trims_whitespace() {
        let input = vec![" chat:write ".to_string(), "  all  ".to_string()];
        let result = expand_scopes(&input);

        assert!(result.contains(&"chat:write".to_string()));
        assert!(result.contains(&"users:read".to_string())); // from "all"
    }

    #[test]
    fn test_expand_scopes_with_bot_all_preset() {
        let input = vec!["bot:all".to_string()];
        let result = expand_scopes(&input);

        assert!(result.contains(&"chat:write".to_string()));
        assert!(result.contains(&"channels:read".to_string()));
        assert!(!result.contains(&"search:read".to_string())); // user-only scope
    }

    #[test]
    fn test_expand_scopes_with_user_all_preset() {
        let input = vec!["user:all".to_string()];
        let result = expand_scopes(&input);

        assert!(result.contains(&"search:read".to_string()));
        assert!(result.contains(&"users:read".to_string()));
        assert!(result.contains(&"chat:write".to_string()));
    }

    #[test]
    fn test_expand_scopes_with_context_bot() {
        let input = vec!["all".to_string()];
        let result = expand_scopes_with_context(&input, true);

        // In bot context, "all" expands to bot scopes
        assert!(result.contains(&"chat:write".to_string()));
        assert!(!result.contains(&"search:read".to_string())); // user-only
    }

    #[test]
    fn test_expand_scopes_with_context_user() {
        let input = vec!["all".to_string()];
        let result = expand_scopes_with_context(&input, false);

        // In user context, "all" expands to user scopes
        assert!(result.contains(&"search:read".to_string()));
        assert!(result.contains(&"users:read".to_string()));
    }

    #[test]
    fn test_bot_all_scopes_no_user_only_scopes() {
        let scopes = bot_all_scopes();
        assert!(!scopes.contains(&"search:read".to_string()));
        assert!(!scopes.contains(&"stars:read".to_string()));
    }

    #[test]
    fn test_user_all_scopes_includes_user_scopes() {
        let scopes = user_all_scopes();
        assert!(scopes.contains(&"search:read".to_string()));
        assert!(scopes.contains(&"stars:read".to_string()));
        assert!(scopes.contains(&"users:read".to_string()));
    }
}
