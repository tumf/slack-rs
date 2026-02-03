# Config and Storage Specification

## Overview
This document defines the exact schema and storage mechanisms for profiles and tokens.

## Config File Location
- Determined by `directories` crate: `ProjectDirs::from("", "", "slackcli")`
- Typical paths:
  - macOS: `~/Library/Application Support/slackcli/profiles.json`
  - Linux: `~/.config/slackcli/profiles.json`
  - Windows: `%APPDATA%\slackcli\profiles.json`

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

## Keyring Storage

### Key Structure
- **Service**: `slackcli`
- **Username**: `{team_id}:{user_id}` (example: `T123ABC:U456DEF`)
- **Secret**: JSON-encoded token data

### Secret Payload Format

```json
{
  "access_token": "xoxp-...",
  "token_type": "user",
  "expires_at": null
}
```

### Field Definitions
- `access_token`: Slack user token (from `oauth.v2.access`)
- `token_type`: Always `"user"` for this CLI
- `expires_at`: Optional expiration timestamp (null for non-expiring tokens)

### Keyring Operations
- **Store**: `keyring::Entry::new("slackcli", "{team_id}:{user_id}").set_password(json_string)`
- **Retrieve**: `keyring::Entry::new("slackcli", "{team_id}:{user_id}").get_password()`
- **Delete**: `keyring::Entry::new("slackcli", "{team_id}:{user_id}").delete_password()`

## Profile Resolution Flow

1. User runs: `slackcli --profile acme-work search "query"`
2. CLI reads `profiles.json`
3. Find profile with `profile_name == "acme-work"`
4. Extract `team_id` and `user_id`
5. Construct keyring key: `{team_id}:{user_id}`
6. Retrieve token from keyring
7. Execute API call with token
8. Update `last_used_at` in `profiles.json`

## Error Handling

### Profile Not Found
- Error: `Profile 'xyz' not found. Run 'slackcli auth list' to see available profiles.`
- Exit code: 1

### Token Not Found in Keyring
- Error: `Token not found for profile 'xyz'. Run 'slackcli auth login --profile xyz' to re-authenticate.`
- Exit code: 1

### Config File Corruption
- Error: `Failed to parse profiles.json: {error}. Consider backing up and deleting the file.`
- Exit code: 1

## Migration Strategy
- If `profiles.json` does not exist: create with `version: 1` and empty `profiles` array
- If `version` field is missing or < 1: attempt to migrate (future-proofing)
- Always validate schema on load; fail fast on corruption
