# Security Specification

## Overview
This document defines security measures to prevent accidental data exposure and destructive operations.

## Token Protection

### Storage
- **NEVER** store tokens in plaintext files
- **ALWAYS** use OS file storage (macOS file storage, Windows Credential Manager, Linux Secret Service)
- Config file (`profiles.json`) must contain only non-secret metadata

### Logging
- **NEVER** log tokens in any form
- Mask `Authorization` headers in debug logs:
  - Before: `Authorization: Bearer xoxp-1234567890-abcdefghij`
  - After: `Authorization: Bearer xoxp-****`
- Mask tokens in error messages:
  - Before: `Failed to authenticate with token xoxp-1234567890`
  - After: `Failed to authenticate with token xoxp-****`

### Output
- **NEVER** include tokens in JSON or text output
- API responses should not echo the token used

## Write Operation Safety

### Two-Tier Protection

#### Tier 1: `--allow-write` Flag (Required for all writes)
- **Scope**: All write operations (`msg post/update/delete`, `react add/remove`)
- **Enforcement**: Check at command execution time
- **Error if missing**:
  ```
  Error: Write operations require --allow-write flag.
  Example: slack-rs --profile myworkspace --allow-write msg post --channel C123 --text "Hello"
  ```
- **Exit code**: 1

#### Tier 2: `--yes` Flag (Required for destructive operations)
- **Scope**: Destructive operations only (`msg delete`, future: `files delete`, etc.)
- **Enforcement**: Check after `--allow-write` validation
- **Behavior if missing**:
  - Display confirmation prompt (localized):
    ```
    You are about to delete a message:
      Channel: #general (C123ABC)
      Timestamp: 1234567890.123456
    
    This action cannot be undone. Continue? [y/N]:
    ```
  - If user enters `y` or `yes` (case-insensitive): proceed
  - Otherwise: abort with exit code 1
- **With `--yes` flag**: Skip prompt, execute immediately

### Implementation Pattern

```rust
fn execute_write_command(allow_write: bool, needs_confirmation: bool, yes: bool) -> Result<()> {
    // Tier 1: Check --allow-write
    if !allow_write {
        return Err(Error::WriteNotAllowed);
    }
    
    // Tier 2: Check confirmation for destructive ops
    if needs_confirmation && !yes {
        if !prompt_user_confirmation()? {
            return Err(Error::OperationCancelled);
        }
    }
    
    // Proceed with operation
    Ok(())
}
```

## Profile Confusion Prevention

### Mandatory `--profile` Flag
- **All API-hitting commands** require explicit `--profile` specification
- **No default profile** (prevents accidental operations on wrong workspace)
- **Error if missing**:
  ```
  Error: --profile is required.
  Run 'slack-rs auth list' to see available profiles.
  ```

### Output Context
- **All JSON responses** must include profile context:
  ```json
  {
    "meta": {
      "profile_name": "acme-work",
      "team_id": "T123ABC",
      "team_name": "Acme Corp",
      "user_id": "U456DEF"
    },
    "data": { ... }
  }
  ```
- **Text output** must show workspace identifier:
  ```
  [acme-work / Acme Corp] Message posted successfully
  ```

## Rate Limiting Protection

### Respect Slack Limits
- Honor HTTP 429 responses
- Parse `Retry-After` header (seconds)
- Wait before retrying

### Exponential Backoff
- Initial retry delay: 1 second
- Multiply by 2 on each retry (with jitter)
- Maximum retry delay: 60 seconds
- Maximum retry attempts: 5

### Jitter Formula
```rust
let jitter = rand::random::<f64>() * 0.3; // ±30%
let delay = base_delay * (1.0 + jitter);
```

## Input Validation

### Channel IDs
- Must match pattern: `^[CDG][A-Z0-9]{8,}$`
- Reject invalid formats early

### Timestamps
- Must match pattern: `^\d{10}\.\d{6}$`
- Example: `1234567890.123456`

### User IDs
- Must match pattern: `^[UW][A-Z0-9]{8,}$`

### Emoji Names
- Must match pattern: `^:[a-z0-9_+-]+:$`
- Example: `:white_check_mark:`

## Error Message Guidelines

### Do Not Expose Sensitive Data
- ❌ Bad: `Failed to post to channel C123ABC with token xoxp-1234567890`
- ✅ Good: `Failed to post to channel C123ABC: missing_scope`

### Provide Actionable Guidance
- ❌ Bad: `Error: 403`
- ✅ Good: `Error: Insufficient permissions. Required scope: chat:write. Run 'slack-rs auth status --profile myworkspace' to check current scopes.`

### Localize User-Facing Messages
- Error messages, prompts, and instructions should respect `--lang` flag
- API error codes remain in English (e.g., `missing_scope`, `channel_not_found`)

## Audit Trail (Future Enhancement)
- Optional audit log for write operations
- Format: JSON lines
- Fields: timestamp, profile, command, channel, result
- Location: `~/.local/share/slack-rs/audit.log`
- Disabled by default; enable with `SLACKRS_AUDIT=1`
