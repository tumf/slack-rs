//! OAuth scope preset definitions and expansion utilities
//!
//! This module provides preset scope definitions (like "all") and utilities
//! to expand them into concrete scope lists.

use std::collections::BTreeSet;

/// Returns the comprehensive "all" scope preset
///
/// This includes common scopes for bot/user tokens but excludes admin/Enterprise-only scopes.
/// The list is stable and sorted alphabetically.
pub fn all_scopes() -> Vec<String> {
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

/// Expands preset names (like "all") in a scope list and removes duplicates
///
/// # Arguments
/// * `input_scopes` - List of scopes which may include preset names like "all" (case-insensitive)
///
/// # Returns
/// A deduplicated, sorted list of concrete scopes with presets expanded
///
/// # Example
/// ```
/// use slack_cli::oauth::scopes::expand_scopes;
///
/// let result = expand_scopes(&["all".to_string(), "custom:scope".to_string()]);
/// assert!(result.contains(&"chat:write".to_string()));
/// assert!(result.contains(&"custom:scope".to_string()));
/// ```
pub fn expand_scopes(input_scopes: &[String]) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for scope in input_scopes {
        let normalized = scope.trim().to_lowercase();

        if normalized == "all" {
            // Expand the "all" preset
            for preset_scope in all_scopes() {
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
}
