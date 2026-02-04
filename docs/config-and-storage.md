# Config and Storage Specification

## Overview
This document defines the exact schema and storage mechanisms for profiles and tokens.

## Config File Location
- All configuration and token data stored in: `~/.config/slack-rs/`
- Platform-independent unified directory structure
- Files:
  - `~/.config/slack-rs/profiles.json` - Profile metadata
  - `~/.config/slack-rs/tokens.json` - Token storage
- Legacy paths (automatically migrated):
  - macOS: `~/Library/Application Support/slack-rs/profiles.json`
  - Linux: `~/.config/slack-rs/profiles.json`
  - Windows: `%APPDATA%\slack-rs\profiles.json`

## profiles.json Schema

```json
{
  "version": 1,
  "profiles": [
    {
      "profile_name": "acme-work",
      "team_id": "T123ABC",
      "team_name": "Acme Corp",
      "user_id": "U456DEF",
      "user_name": "john.doe",
      "scopes": [
        "search:read",
        "channels:read",
        "channels:history",
        "users:read",
        "chat:write"
      ],
      "created_at": "2026-02-03T10:30:00Z",
      "last_used_at": "2026-02-03T15:45:00Z"
    }
  ]
}
```

### Field Definitions
- `version`: Schema version (currently `1`)
- `profiles`: Array of profile objects
  - `profile_name`: User-chosen alias (must be unique)
  - `team_id`: Slack workspace ID (from `oauth.v2.access`)
  - `team_name`: Slack workspace name (from `oauth.v2.access`)
  - `user_id`: Authenticated user ID (from `oauth.v2.access`)
  - `user_name`: Optional user name (can be fetched via `users.info` later)
  - `scopes`: Array of granted OAuth scopes
  - `created_at`: ISO 8601 timestamp of profile creation
  - `last_used_at`: ISO 8601 timestamp of last command execution

### Constraints
- `profile_name` must be unique across all profiles
- `(team_id, user_id)` combination should be unique (enforced on login)
- If a duplicate `(team_id, user_id)` is detected during login:
  - Update existing profile's token and metadata
  - Do not create a new profile entry

## File-Based Token Storage

### Storage Location
- File: `~/.config/slack-rs/tokens.json`
- Format: JSON key-value mapping
- File permissions: `0600` (owner read/write only) on Unix systems

### tokens.json Format

```json
{
  "T123ABC:U456DEF": "xoxp-XXXXXXXXXX-XXXXXXXXXX-XXXXXXXXXXXX-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "T123ABC:U456DEF:user": "xoxp-...",
  "T789GHI:U012JKL": "xoxb-...",
  "oauth-client-secret:default": "abc123def456..."
}
```

### Key Structure
- **Profile tokens**: `{team_id}:{user_id}` (example: `T123ABC:U456DEF`)
- **Scoped tokens**: `{team_id}:{user_id}:{scope}` (example: `T123ABC:U456DEF:user`)
- **OAuth secrets**: `oauth-client-secret:{profile_name}` (example: `oauth-client-secret:default`)

### Token Operations
- **Store**: Write key-value pair to JSON file with 0600 permissions
- **Retrieve**: Read token value by key from JSON file
- **Delete**: Remove key from JSON file
- **List**: Return all keys in JSON file

### Security Considerations
- File permissions set to `0600` (Unix) to prevent unauthorized access
- `tokens.json` included in `.gitignore` to prevent accidental commits
- No encryption applied - security relies on filesystem permissions
- Tokens stored in plaintext within the file
- Consider using system keyring for enhanced security in future versions

## Profile Resolution Flow

1. User runs: `slack-rs --profile acme-work search "query"`
2. CLI reads `~/.config/slack-rs/profiles.json`
3. Find profile with `profile_name == "acme-work"`
4. Extract `team_id` and `user_id`
5. Construct token key: `{team_id}:{user_id}`
6. Retrieve token from `~/.config/slack-rs/tokens.json` file
7. Execute API call with token
8. Update `last_used_at` in `profiles.json`

## Error Handling

### Profile Not Found
- Error: `Profile 'xyz' not found. Run 'slack-rs auth list' to see available profiles.`
- Exit code: 1

### Token Not Found in Storage
- Error: `Token not found for profile 'xyz'. Run 'slack-rs auth login --profile xyz' to re-authenticate.`
- Exit code: 1

### Config File Corruption
- Error: `Failed to parse profiles.json: {error}. Consider backing up and deleting the file.`
- Exit code: 1

### Token File Corruption
- Error: `Failed to parse tokens.json: {error}. Consider backing up and deleting the file.`
- Exit code: 1

### File Permission Issues
- Error: `Failed to set secure permissions on tokens.json: {error}`
- Warning: Tokens file may be readable by other users
- Exit code: 0 (warning only, operation continues)

## Migration Strategy
- If `profiles.json` does not exist at the new path: check for legacy config and migrate
  - Legacy config path: `ProjectDirs::from("", "", "slack-rs")` + `profiles.json`
  - Migration: Try `fs::rename` first; if it fails, copy content and keep old file
  - Migration is automatic and transparent on first access
- If `profiles.json` does not exist: create with `version: 1` and empty `profiles` array
- If `version` field is missing or < 1: attempt to migrate (future-proofing)
- Always validate schema on load; fail fast on corruption
