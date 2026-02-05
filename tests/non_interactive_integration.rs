//! Integration tests for non-interactive mode

use slack_rs::api::ApiError;
use slack_rs::commands::guards::confirm_destructive;

/// Test that confirm_destructive requires --yes in non-interactive mode
#[test]
fn test_confirm_destructive_non_interactive_without_yes() {
    let result = confirm_destructive(false, "delete message", true);
    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::NonInteractiveError(msg) => {
            assert!(msg.contains("Use --yes flag"));
        }
        _ => panic!("Expected NonInteractiveError"),
    }
}

/// Test that confirm_destructive succeeds with --yes in non-interactive mode
#[test]
fn test_confirm_destructive_non_interactive_with_yes() {
    let result = confirm_destructive(true, "delete message", true);
    assert!(result.is_ok());
}

/// Test that confirm_destructive works normally in interactive mode with --yes
#[test]
fn test_confirm_destructive_interactive_with_yes() {
    let result = confirm_destructive(true, "delete message", false);
    assert!(result.is_ok());
}
