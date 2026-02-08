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
                FlagDef {
                    name: "--idempotency-key".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Idempotency key for preventing duplicate operations".to_string(),
                    default: None,
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
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--idempotency-key".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Idempotency key for preventing duplicate operations".to_string(),
                    default: None,
                },
            ],
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
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--idempotency-key".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Idempotency key for preventing duplicate operations".to_string(),
                    default: None,
                },
            ],
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
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--idempotency-key".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Idempotency key for preventing duplicate operations".to_string(),
                    default: None,
                },
            ],
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
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--idempotency-key".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Idempotency key for preventing duplicate operations".to_string(),
                    default: None,
                },
            ],
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
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--idempotency-key".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Idempotency key for preventing duplicate operations".to_string(),
                    default: None,
                },
            ],
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
        // file download
        CommandDef {
            name: "file download".to_string(),
            description: "Download a file from Slack".to_string(),
            usage: "slack-rs file download [<file_id>] [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--url".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Direct download URL (alternative to file_id)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--out".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Output path (omit for current directory, '-' for stdout, directory for auto-naming)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--token-type".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Token type (bot or user)".to_string(),
                    default: None,
                },
            ],
            examples: vec![
                ExampleDef {
                    description: "Download by file ID".to_string(),
                    command: "slack-rs file download F123456".to_string(),
                },
                ExampleDef {
                    description: "Download to stdout".to_string(),
                    command: "slack-rs file download F123456 --out -".to_string(),
                },
                ExampleDef {
                    description: "Download by URL".to_string(),
                    command: "slack-rs file download --url https://files.slack.com/...".to_string(),
                },
            ],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Download failed".to_string(),
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
        // auth rename
        CommandDef {
            name: "auth rename".to_string(),
            description: "Rename a profile".to_string(),
            usage: "slack-rs auth rename <old_name> <new_name>".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "Rename profile".to_string(),
                command: "slack-rs auth rename work personal".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Rename failed".to_string(),
                },
            ],
        },
        // auth export
        CommandDef {
            name: "auth export".to_string(),
            description: "Export profiles to encrypted file".to_string(),
            usage: "slack-rs auth export [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Export specific profile".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--all".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Export all profiles".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--out".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "Output file path".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--passphrase-env".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Environment variable containing passphrase".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--passphrase-prompt".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Prompt for passphrase".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--yes".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Confirm dangerous operation".to_string(),
                    default: None,
                },
            ],
            examples: vec![ExampleDef {
                description: "Export all profiles".to_string(),
                command: "slack-rs auth export --all --out profiles.enc --yes".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Export failed".to_string(),
                },
            ],
        },
        // auth import
        CommandDef {
            name: "auth import".to_string(),
            description: "Import profiles from encrypted file".to_string(),
            usage: "slack-rs auth import [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--in".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "Input file path".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--passphrase-env".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Environment variable containing passphrase".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--passphrase-prompt".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Prompt for passphrase".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--yes".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Automatically accept conflicts".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--force".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Overwrite existing profiles".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--dry-run".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Show what would be imported without making changes".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--json".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Output results in JSON format".to_string(),
                    default: None,
                },
            ],
            examples: vec![
                ExampleDef {
                    description: "Import profiles".to_string(),
                    command: "slack-rs auth import --in profiles.enc".to_string(),
                },
                ExampleDef {
                    description: "Preview import without making changes".to_string(),
                    command: "slack-rs auth import --in profiles.enc --dry-run".to_string(),
                },
                ExampleDef {
                    description: "Preview import with JSON output".to_string(),
                    command: "slack-rs auth import --in profiles.enc --dry-run --json".to_string(),
                },
            ],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Import failed".to_string(),
                },
            ],
        },
        // config oauth set
        CommandDef {
            name: "config oauth set".to_string(),
            description: "Set OAuth configuration for a profile".to_string(),
            usage: "slack-rs config oauth set <profile> --client-id <id> --redirect-uri <uri> --scopes <scopes> [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--client-id".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "OAuth client ID".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--redirect-uri".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "OAuth redirect URI".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--scopes".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "Comma-separated list of scopes or 'all'".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--client-secret-env".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Read secret from environment variable".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--client-secret-file".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Read secret from file".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--client-secret".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Direct secret value (requires --yes, unsafe)".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--yes".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Confirm dangerous operation".to_string(),
                    default: None,
                },
            ],
            examples: vec![ExampleDef {
                description: "Set OAuth config".to_string(),
                command: "slack-rs config oauth set work --client-id 123.456 --redirect-uri http://127.0.0.1:8765/callback --scopes all".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Config set failed".to_string(),
                },
            ],
        },
        // config oauth show
        CommandDef {
            name: "config oauth show".to_string(),
            description: "Show OAuth configuration for a profile".to_string(),
            usage: "slack-rs config oauth show <profile>".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "Show OAuth config".to_string(),
                command: "slack-rs config oauth show work".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Config show failed".to_string(),
                },
            ],
        },
        // config oauth delete
        CommandDef {
            name: "config oauth delete".to_string(),
            description: "Delete OAuth configuration for a profile".to_string(),
            usage: "slack-rs config oauth delete <profile>".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "Delete OAuth config".to_string(),
                command: "slack-rs config oauth delete work".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Config delete failed".to_string(),
                },
            ],
        },
        // config set
        CommandDef {
            name: "config set".to_string(),
            description: "Set default token type for a profile".to_string(),
            usage: "slack-rs config set <profile> --token-type <type>".to_string(),
            flags: vec![FlagDef {
                name: "--token-type".to_string(),
                flag_type: "string".to_string(),
                required: true,
                description: "Default token type (bot or user)".to_string(),
                default: None,
            }],
            examples: vec![ExampleDef {
                description: "Set token type".to_string(),
                command: "slack-rs config set work --token-type bot".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Config set failed".to_string(),
                },
            ],
        },
        // conv select
        CommandDef {
            name: "conv select".to_string(),
            description: "Interactively select a conversation".to_string(),
            usage: "slack-rs conv select [flags]".to_string(),
            flags: vec![FlagDef {
                name: "--profile".to_string(),
                flag_type: "string".to_string(),
                required: false,
                description: "Profile name".to_string(),
                default: Some("default".to_string()),
            }],
            examples: vec![ExampleDef {
                description: "Select conversation".to_string(),
                command: "slack-rs conv select".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Selection failed".to_string(),
                },
            ],
        },
        // users cache-update
        CommandDef {
            name: "users cache-update".to_string(),
            description: "Update user cache for mention resolution".to_string(),
            usage: "slack-rs users cache-update [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--force".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Force cache update".to_string(),
                    default: None,
                },
            ],
            examples: vec![ExampleDef {
                description: "Update user cache".to_string(),
                command: "slack-rs users cache-update".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Cache update failed".to_string(),
                },
            ],
        },
        // users resolve-mentions
        CommandDef {
            name: "users resolve-mentions".to_string(),
            description: "Resolve user mentions in text".to_string(),
            usage: "slack-rs users resolve-mentions <text> [flags]".to_string(),
            flags: vec![
                FlagDef {
                    name: "--profile".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Profile name".to_string(),
                    default: Some("default".to_string()),
                },
                FlagDef {
                    name: "--format".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Output format".to_string(),
                    default: None,
                },
            ],
            examples: vec![ExampleDef {
                description: "Resolve mentions".to_string(),
                command: "slack-rs users resolve-mentions '@john said hello'".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Resolution failed".to_string(),
                },
            ],
        },
        // commands
        CommandDef {
            name: "commands".to_string(),
            description: "List all available commands in machine-readable format".to_string(),
            usage: "slack-rs commands --json".to_string(),
            flags: vec![FlagDef {
                name: "--json".to_string(),
                flag_type: "boolean".to_string(),
                required: true,
                description: "Output in JSON format".to_string(),
                default: None,
            }],
            examples: vec![ExampleDef {
                description: "List commands".to_string(),
                command: "slack-rs commands --json".to_string(),
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
        // schema
        CommandDef {
            name: "schema".to_string(),
            description: "Show output schema for a command".to_string(),
            usage: "slack-rs schema --command <cmd> --output json-schema".to_string(),
            flags: vec![
                FlagDef {
                    name: "--command".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "Command name".to_string(),
                    default: None,
                },
                FlagDef {
                    name: "--output".to_string(),
                    flag_type: "string".to_string(),
                    required: true,
                    description: "Output format (json-schema)".to_string(),
                    default: None,
                },
            ],
            examples: vec![ExampleDef {
                description: "Show schema".to_string(),
                command: "slack-rs schema --command conv.list --output json-schema".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Schema generation failed".to_string(),
                },
            ],
        },
        // install-skills
        CommandDef {
            name: "install-skills".to_string(),
            description: "Install agent skill from embedded or local source".to_string(),
            usage: "slack-rs install-skills [source] [--global]".to_string(),
            flags: vec![
                FlagDef {
                    name: "source".to_string(),
                    flag_type: "string".to_string(),
                    required: false,
                    description: "Source to install from: 'self' (embedded) or 'local:<path>'".to_string(),
                    default: Some("self".to_string()),
                },
                FlagDef {
                    name: "--global".to_string(),
                    flag_type: "boolean".to_string(),
                    required: false,
                    description: "Install to ~/.agents instead of ./.agents".to_string(),
                    default: Some("false".to_string()),
                },
            ],
            examples: vec![
                ExampleDef {
                    description: "Install embedded skill (default)".to_string(),
                    command: "slack-rs install-skills".to_string(),
                },
                ExampleDef {
                    description: "Install from local path".to_string(),
                    command: "slack-rs install-skills local:/path/to/skill".to_string(),
                },
                ExampleDef {
                    description: "Install globally to ~/.agents".to_string(),
                    command: "slack-rs install-skills --global".to_string(),
                },
            ],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success - skill installed".to_string(),
                },
                ExitCodeDef {
                    code: 1,
                    description: "Failure - installation error".to_string(),
                },
            ],
        },
        // demo
        CommandDef {
            name: "demo".to_string(),
            description: "Run demonstration".to_string(),
            usage: "slack-rs demo".to_string(),
            flags: vec![],
            examples: vec![ExampleDef {
                description: "Run demo".to_string(),
                command: "slack-rs demo".to_string(),
            }],
            exit_codes: vec![
                ExitCodeDef {
                    code: 0,
                    description: "Success".to_string(),
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

    // Special case for install-skills
    let schema = if command_name == "install-skills" {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "schemaVersion": {
                    "type": "string",
                    "description": "Schema version number"
                },
                "type": {
                    "type": "string",
                    "description": "Response type identifier",
                    "const": "skill-installation"
                },
                "ok": {
                    "type": "boolean",
                    "description": "Indicates if the operation was successful"
                },
                "skills": {
                    "type": "array",
                    "description": "List of installed skills",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Skill name"
                            },
                            "path": {
                                "type": "string",
                                "description": "Installation path"
                            },
                            "source_type": {
                                "type": "string",
                                "description": "Source type (self or local)"
                            }
                        },
                        "required": ["name", "path", "source_type"]
                    }
                }
            },
            "required": ["schemaVersion", "type", "ok", "skills"]
        })
    } else {
        // Generate basic envelope schema for other commands
        serde_json::json!({
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
        })
    };

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
