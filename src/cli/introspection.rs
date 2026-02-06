//! CLI introspection - self-describing CLI capabilities
//!
//! Provides machine-readable information about commands, flags, and output schemas.
//! Implements Agentic CLI Design principle 7 (Introspectable).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// CLI command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDef {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub flags: Vec<FlagDef>,
    pub examples: Vec<ExampleDef>,
    pub exit_codes: Vec<ExitCodeDef>,
}

/// Flag/option definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDef {
    pub name: String,
    #[serde(rename = "type")]
    pub flag_type: String,
    pub required: bool,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Command example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleDef {
    pub description: String,
    pub command: String,
}

/// Exit code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodeDef {
    pub code: i32,
    pub description: String,
}

/// Commands list response
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandsListResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub ok: bool,
    pub commands: Vec<CommandDef>,
}

/// Structured help response
#[derive(Debug, Serialize, Deserialize)]
pub struct HelpResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub ok: bool,
    pub command: String,
    pub usage: String,
    pub flags: Vec<FlagDef>,
    pub examples: Vec<ExampleDef>,
    #[serde(rename = "exitCodes")]
    pub exit_codes: Vec<ExitCodeDef>,
}

/// Schema response
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub ok: bool,
    pub command: String,
    pub schema: Value,
}

/// Get all command definitions
pub fn get_command_definitions() -> Vec<CommandDef> {
    vec![
        // api call
        CommandDef {
            name: "api call".to_string(),
            description: "Call a Slack API method".to_string(),
            usage: "slack-rs api call <method> [key=value]... [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--json".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Send as JSON body (default: form-urlencoded)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--get".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Use GET method (default: POST)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--raw".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Output raw Slack API response (without envelope)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
            ],
            examples: vec![
                ExampleDef {
                    description: "Get user info".to_string(),
                    command: "slack-rs api call users.info user=U123456 --get".to_string(),
                },
                ExampleDef {
                    description: "Post message".to_string(),
                    command: "slack-rs api call chat.postMessage channel=C123 text=Hello"
                        .to_string(),
                },
            ],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "API call failed".to_string(),
                },
            ],
        },
        // auth login
        CommandDef {
            name: "auth login".to_string(),
            description: "Authenticate with Slack via OAuth".to_string(),
            usage: "slack-rs auth login [profile_name] [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--client-id".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "OAuth client ID".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--bot-scopes".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Bot scopes (comma-separated or 'all')".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--user-scopes".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "User scopes (comma-separated or 'all')".to_string(),
                    default: None,
                },
            ],
            examples: vec![ExampleDef {
                description: "Login with default profile".to_string(),
                command: "slack-rs auth login".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Login failed".to_string(),
                },
            ],
        },
        // auth status
        CommandDef {
            name: "auth status".to_string(),
            description: "Show authentication status".to_string(),
            usage: "slack-rs auth status [profile_name]".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "Check status".to_string(),
                command: "slack-rs auth status".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // auth list
        CommandDef {
            name: "auth list".to_string(),
            description: "List all profiles".to_string(),
            usage: "slack-rs auth list".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "List profiles".to_string(),
                command: "slack-rs auth list".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // auth logout
        CommandDef {
            name: "auth logout".to_string(),
            description: "Remove authentication for a profile".to_string(),
            usage: "slack-rs auth logout [profile_name]".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "Logout".to_string(),
                command: "slack-rs auth logout".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // conv list
        CommandDef {
            name: "conv list".to_string(),
            description: "List conversations".to_string(),
            usage: "slack-rs conv list [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--types".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Conversation types (comma-separated)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--limit".to_string(),
                    flag_type: "integer".to_string(),
                    required: false,
                    description: "Maximum number of conversations".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--filter".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Filter (key:value format, can be repeated)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--format".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Output format (json, jsonl, table, tsv)".to_string(),
                    default: Some("json".to_string()),
                },
                FlagDef {
                    name: "--raw".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Output raw response (without envelope)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
            ],
            examples: vec![
                ExampleDef {
                    description: "List all conversations".to_string(),
                    command: "slack-rs conv list".to_string(),
                },
                ExampleDef {
                    description: "List with filter".to_string(),
                    command: "slack-rs conv list --filter is_member:true".to_string(),
                },
            ],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "API call failed".to_string(),
                },
            ],
        },
        // conv search
        CommandDef {
            name: "conv search".to_string(),
            description: "Search conversations by name".to_string(),
            usage: "slack-rs conv search <pattern> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Search conversations".to_string(),
                command: "slack-rs conv search general".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // conv history
        CommandDef {
            name: "conv history".to_string(),
            description: "Get conversation history".to_string(),
            usage: "slack-rs conv history <channel> [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--limit".to_string(),
                    flag_type: "integer".to_string(),
                    required: false,
                    description: "Maximum number of messages".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
            ],
            examples: vec![ExampleDef {
                description: "Get history".to_string(),
                command: "slack-rs conv history C123456".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // msg post
        CommandDef {
            name: "msg post".to_string(),
            description: "Post a message to a channel".to_string(),
            usage: "slack-rs msg post <channel> <text> [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--thread-ts".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Thread timestamp for reply".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--reply-broadcast".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Broadcast reply to channel".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
            ],
            examples: vec![ExampleDef {
                description: "Post message".to_string(),
                command: "slack-rs msg post C123 'Hello world'".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Post failed".to_string(),
                },
            ],
        },
        // msg update
        CommandDef {
            name: "msg update".to_string(),
            description: "Update a message".to_string(),
            usage: "slack-rs msg update <channel> <ts> <text> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Update message".to_string(),
                command: "slack-rs msg update C123 1234567890.123456 'Updated text'".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Update failed".to_string(),
                },
            ],
        },
        // msg delete
        CommandDef {
            name: "msg delete".to_string(),
            description: "Delete a message".to_string(),
            usage: "slack-rs msg delete <channel> <ts> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Delete message".to_string(),
                command: "slack-rs msg delete C123 1234567890.123456".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Delete failed".to_string(),
                },
            ],
        },
        // users info
        CommandDef {
            name: "users info".to_string(),
            description: "Get user information".to_string(),
            usage: "slack-rs users info <user_id> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Get user info".to_string(),
                command: "slack-rs users info U123456".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // react add
        CommandDef {
            name: "react add".to_string(),
            description: "Add a reaction to a message".to_string(),
            usage: "slack-rs react add <channel> <ts> <emoji> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Add reaction".to_string(),
                command: "slack-rs react add C123 1234567890.123456 thumbsup".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // react remove
        CommandDef {
            name: "react remove".to_string(),
            description: "Remove a reaction from a message".to_string(),
            usage: "slack-rs react remove <channel> <ts> <emoji> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Remove reaction".to_string(),
                command: "slack-rs react remove C123 1234567890.123456 thumbsup".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Command failed".to_string(),
                },
            ],
        },
        // file upload
        CommandDef {
            name: "file upload".to_string(),
            description: "Upload a file".to_string(),
            usage: "slack-rs file upload <path> [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Upload file".to_string(),
                command: "slack-rs file upload document.pdf".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Upload failed".to_string(),
                },
            ],
        },
        // search
        CommandDef {
            name: "search".to_string(),
            description: "Search messages".to_string(),
            usage: "slack-rs search <query> [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--count".to_string(),
                    flag_type: "integer".to_string(),
                    required: false,
                    description: "Number of results".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--page".to_string(),
                    flag_type: "integer".to_string(),
                    required: false,
                    description: "Page number".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
            ],
            examples: vec![ExampleDef {
                description: "Search messages".to_string(),
                command: "slack-rs search 'important announcement'".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Search failed".to_string(),
                },
            ],
        },
    ]
}

/// Normalize command name (converts "conv.list" to "conv list", etc.)
fn normalize_command_name(name: &str) -> String {
    // Replace dots with spaces for consistent lookup
    name.replace('.', " ")
}

/// Get command definition by name
/// Supports both space-separated ("conv list") and dot-separated ("conv.list") formats
pub fn get_command_definition(command_name: &str) -> Option<CommandDef> {
    let normalized = normalize_command_name(command_name);
    get_command_definitions()
        .into_iter()
        .find(|cmd| cmd.name == normalized)
}

/// Generate commands list response
pub fn generate_commands_list() -> CommandsListResponse {
    CommandsListResponse {
        schema_version: 1,
        response_type: "commands.list".to_string(),
        ok: true,
        commands: get_command_definitions(),
    }
}

/// Generate structured help for a command
pub fn generate_help(command_name: &str) -> Result<HelpResponse, String> {
    let cmd = get_command_definition(command_name)
        .ok_or_else(|| format!("Command '{}' not found", command_name))?;

    Ok(HelpResponse {
        schema_version: 1,
        response_type: "help".to_string(),
        ok: true,
        command: cmd.name.clone(),
        usage: cmd.usage.clone(),
        flags: cmd.flags.clone(),
        examples: cmd.examples.clone(),
        exit_codes: cmd.exit_codes.clone(),
    })
}

/// Generate JSON schema for a command's output
pub fn generate_schema(command_name: &str) -> Result<SchemaResponse, String> {
    // Verify command exists
    let _cmd = get_command_definition(command_name)
        .ok_or_else(|| format!("Command '{}' not found", command_name))?;

    // Generate basic envelope schema
    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "schemaVersion": {
                "type": "integer",
                "description": "Schema version number"
            },
            "type": {
                "type": "string",
                "description": "Response type identifier"
            },
            "ok": {
                "type": "boolean",
                "description": "Indicates if the operation was successful"
            },
            "response": {
                "type": "object",
                "description": "Slack API response data"
            },
            "meta": {
                "type": "object",
                "description": "Metadata about the request and profile",
                "properties": {
                    "profile": {"type": "string"},
                    "team_id": {"type": "string"},
                    "user_id": {"type": "string"},
                    "method": {"type": "string"},
                    "command": {"type": "string"}
                }
            }
        },
        "required": ["schemaVersion", "type", "ok"]
    });

    Ok(SchemaResponse {
        schema_version: 1,
        response_type: "schema".to_string(),
        ok: true,
        command: command_name.to_string(),
        schema,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_definitions() {
        let commands = get_command_definitions();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "api call"));
        assert!(commands.iter().any(|c| c.name == "conv list"));
    }

    #[test]
    fn test_get_command_definition() {
        let cmd = get_command_definition("conv list");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.name, "conv list");
        assert!(!cmd.flags.is_empty());
    }

    #[test]
    fn test_generate_commands_list() {
        let response = generate_commands_list();
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.response_type, "commands.list");
        assert!(response.ok);
        assert!(!response.commands.is_empty());
    }

    #[test]
    fn test_generate_help() {
        let help = generate_help("conv list");
        assert!(help.is_ok());
        let help = help.unwrap();
        assert_eq!(help.schema_version, 1);
        assert_eq!(help.response_type, "help");
        assert!(help.ok);
        assert_eq!(help.command, "conv list");
    }

    #[test]
    fn test_generate_help_unknown_command() {
        let help = generate_help("unknown command");
        assert!(help.is_err());
    }

    #[test]
    fn test_generate_schema() {
        let schema = generate_schema("conv list");
        assert!(schema.is_ok());
        let schema = schema.unwrap();
        assert_eq!(schema.schema_version, 1);
        assert_eq!(schema.response_type, "schema");
        assert!(schema.ok);
        assert_eq!(schema.command, "conv list");
    }

    #[test]
    fn test_commands_list_json_serialization() {
        let response = generate_commands_list();
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        // Verify it can be parsed back
        let parsed: Result<CommandsListResponse, _> = serde_json::from_str(&json.unwrap());
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_help_json_serialization() {
        let help = generate_help("conv list").unwrap();
        let json = serde_json::to_string(&help);
        assert!(json.is_ok());

        // Verify it can be parsed back
        let parsed: Result<HelpResponse, _> = serde_json::from_str(&json.unwrap());
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_schema_json_serialization() {
        let schema = generate_schema("conv list").unwrap();
        let json = serde_json::to_string(&schema);
        assert!(json.is_ok());

        // Verify it can be parsed back
        let parsed: Result<SchemaResponse, _> = serde_json::from_str(&json.unwrap());
        assert!(parsed.is_ok());
    }
}
