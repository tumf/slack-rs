use slack_rs::oauth::{all_scopes, expand_scopes};

#[test]
fn test_all_preset_expands_to_comprehensive_list() {
    let input = vec!["all".to_string()];
    let result = expand_scopes(&input);

    // Verify it contains the expected core scopes
    assert!(result.contains(&"chat:write".to_string()));
    assert!(result.contains(&"users:read".to_string()));
    assert!(result.contains(&"channels:read".to_string()));
    assert!(result.contains(&"files:read".to_string()));

    // Verify the count is reasonable (should be 30+ scopes)
    assert!(
        result.len() >= 30,
        "Expected at least 30 scopes in 'all' preset"
    );
}

#[test]
fn test_all_preset_case_insensitive() {
    let lowercase = expand_scopes(&["all".to_string()]);
    let uppercase = expand_scopes(&["ALL".to_string()]);
    let mixed = expand_scopes(&["AlL".to_string()]);

    assert_eq!(lowercase, uppercase);
    assert_eq!(uppercase, mixed);
}

#[test]
fn test_all_preset_mixed_with_custom_scopes() {
    let input = vec!["all".to_string(), "custom:scope".to_string()];
    let result = expand_scopes(&input);

    // Should contain all preset scopes
    assert!(result.contains(&"chat:write".to_string()));
    assert!(result.contains(&"users:read".to_string()));

    // Should also contain the custom scope
    assert!(result.contains(&"custom:scope".to_string()));
}

#[test]
fn test_all_preset_deduplicates_overlaps() {
    let input = vec![
        "all".to_string(),
        "chat:write".to_string(), // This is in "all"
    ];
    let result = expand_scopes(&input);

    // chat:write should appear only once
    let count = result.iter().filter(|s| *s == "chat:write").count();
    assert_eq!(
        count, 1,
        "chat:write should appear only once after deduplication"
    );
}

#[test]
fn test_regular_scopes_without_preset() {
    let input = vec!["chat:write".to_string(), "users:read".to_string()];
    let result = expand_scopes(&input);

    assert_eq!(result.len(), 2);
    assert!(result.contains(&"chat:write".to_string()));
    assert!(result.contains(&"users:read".to_string()));
}

#[test]
fn test_all_scopes_function_returns_stable_list() {
    let scopes1 = all_scopes();
    let scopes2 = all_scopes();

    assert_eq!(scopes1, scopes2);
    assert!(!scopes1.is_empty());
}

#[test]
fn test_whitespace_handling() {
    let input = vec![" all ".to_string(), "  chat:write  ".to_string()];
    let result = expand_scopes(&input);

    // Should handle whitespace and expand "all"
    assert!(result.contains(&"users:read".to_string())); // from "all"
    assert!(result.contains(&"chat:write".to_string())); // direct + from "all"
}
