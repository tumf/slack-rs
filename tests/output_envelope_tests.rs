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

    // Verify introspection fields
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["type"], "conversations.list");
    assert_eq!(json["ok"], true);

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
        token_type: Some("bot".to_string()),
        idempotency_key: None,
        idempotency_status: None,
    };

    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: CommandMeta = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.profile_name, Some("test".to_string()));
    assert_eq!(deserialized.team_id, "T999");
    assert_eq!(deserialized.user_id, "U888");
    assert_eq!(deserialized.method, "chat.postMessage");
    assert_eq!(deserialized.command, "msg post");
    assert_eq!(deserialized.token_type, Some("bot".to_string()));
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
        token_type: Some("bot".to_string()),
        idempotency_key: None,
        idempotency_status: None,
    };

    let api_json = serde_json::to_value(&api_meta).unwrap();
    let wrapper_json = serde_json::to_value(&wrapper_meta).unwrap();

    // Both should have command field
    assert_eq!(api_json["command"], "api call");
    assert_eq!(wrapper_json["command"], "conv list");

    // Both should have the same method
    assert_eq!(api_json["method"], wrapper_json["method"]);
}

#[test]
fn test_command_response_with_token_type() {
    let response = CommandResponse::with_token_type(
        json!({"ok": true, "channels": []}),
        Some("default".to_string()),
        "T123ABC".to_string(),
        "U456DEF".to_string(),
        "conversations.list".to_string(),
        "conv list".to_string(),
        Some("bot".to_string()),
    );

    let json = serde_json::to_value(&response).unwrap();

    // Verify introspection fields
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["type"], "conversations.list");
    assert_eq!(json["ok"], true);

    // Verify envelope structure
    assert!(json["response"].is_object());
    assert!(json["meta"].is_object());

    // Verify meta fields including token_type
    assert_eq!(json["meta"]["profile_name"], "default");
    assert_eq!(json["meta"]["team_id"], "T123ABC");
    assert_eq!(json["meta"]["user_id"], "U456DEF");
    assert_eq!(json["meta"]["method"], "conversations.list");
    assert_eq!(json["meta"]["command"], "conv list");
    assert_eq!(json["meta"]["token_type"], "bot");
}

#[test]
fn test_command_response_without_token_type_omits_field() {
    let response = CommandResponse::with_token_type(
        json!({"ok": true}),
        Some("default".to_string()),
        "T123".to_string(),
        "U456".to_string(),
        "users.info".to_string(),
        "users info".to_string(),
        None,
    );

    let json_str = serde_json::to_string(&response).unwrap();
    // token_type should not be present in JSON when None
    assert!(!json_str.contains("token_type"));
}

#[test]
fn test_command_response_with_user_token_type() {
    let response = CommandResponse::with_token_type(
        json!({"ok": true}),
        Some("default".to_string()),
        "T123".to_string(),
        "U456".to_string(),
        "users.info".to_string(),
        "users info".to_string(),
        Some("user".to_string()),
    );

    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["meta"]["token_type"], "user");
}
