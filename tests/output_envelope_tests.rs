//! Tests for unified output envelope and --raw flag

use serde_json::json;
use slack_rs::api::{ApiCallMeta, ApiCallResponse, CommandMeta, CommandResponse};

#[test]
fn test_api_call_meta_includes_command() {
    let meta = ApiCallMeta {
        profile_name: Some("default".to_string()),
        team_id: "T123ABC".to_string(),
        user_id: "U456DEF".to_string(),
        method: "conversations.list".to_string(),
        command: "api call".to_string(),
        token_type: "bot".to_string(),
    };

    let json = serde_json::to_value(&meta).unwrap();
    assert_eq!(json["command"], "api call");
    assert_eq!(json["method"], "conversations.list");
}

#[test]
fn test_command_response_envelope() {
    let response = CommandResponse::new(
        json!({"ok": true, "channels": []}),
        Some("default".to_string()),
        "T123ABC".to_string(),
        "U456DEF".to_string(),
        "conversations.list".to_string(),
        "conv list".to_string(),
    );

    let json = serde_json::to_value(&response).unwrap();

    // Verify envelope structure
    assert!(json["response"].is_object());
    assert!(json["meta"].is_object());

    // Verify meta fields
    assert_eq!(json["meta"]["profile_name"], "default");
    assert_eq!(json["meta"]["team_id"], "T123ABC");
    assert_eq!(json["meta"]["user_id"], "U456DEF");
    assert_eq!(json["meta"]["method"], "conversations.list");
    assert_eq!(json["meta"]["command"], "conv list");
}

#[test]
fn test_api_call_response_with_command() {
    let response = ApiCallResponse {
        response: json!({
            "ok": true,
            "channels": []
        }),
        meta: ApiCallMeta {
            profile_name: Some("work".to_string()),
            team_id: "T123ABC".to_string(),
            user_id: "U456DEF".to_string(),
            method: "conversations.list".to_string(),
            command: "api call".to_string(),
            token_type: "bot".to_string(),
        },
    };

    let json = serde_json::to_value(&response).unwrap();

    // Verify structure
    assert!(json["response"]["ok"].as_bool().unwrap());
    assert_eq!(json["meta"]["command"], "api call");
    assert_eq!(json["meta"]["method"], "conversations.list");
    assert_eq!(json["meta"]["token_type"], "bot");
}

#[test]
fn test_command_meta_serialization() {
    let meta = CommandMeta {
        profile_name: Some("test".to_string()),
        team_id: "T999".to_string(),
        user_id: "U888".to_string(),
        method: "chat.postMessage".to_string(),
        command: "msg post".to_string(),
    };

    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: CommandMeta = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.profile_name, Some("test".to_string()));
    assert_eq!(deserialized.team_id, "T999");
    assert_eq!(deserialized.user_id, "U888");
    assert_eq!(deserialized.method, "chat.postMessage");
    assert_eq!(deserialized.command, "msg post");
}

#[test]
fn test_different_commands_have_different_command_names() {
    // Test api call
    let api_meta = ApiCallMeta {
        profile_name: Some("default".to_string()),
        team_id: "T123".to_string(),
        user_id: "U123".to_string(),
        method: "conversations.list".to_string(),
        command: "api call".to_string(),
        token_type: "bot".to_string(),
    };

    // Test wrapper command
    let wrapper_meta = CommandMeta {
        profile_name: Some("default".to_string()),
        team_id: "T123".to_string(),
        user_id: "U123".to_string(),
        method: "conversations.list".to_string(),
        command: "conv list".to_string(),
    };

    let api_json = serde_json::to_value(&api_meta).unwrap();
    let wrapper_json = serde_json::to_value(&wrapper_meta).unwrap();

    // Both should have command field
    assert_eq!(api_json["command"], "api call");
    assert_eq!(wrapper_json["command"], "conv list");

    // Both should have the same method
    assert_eq!(api_json["method"], wrapper_json["method"]);
}
