# Command Specification

## Global Flags

All commands support these global flags:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--profile <name>` | String | (required) | Profile name to use |
| `--format <fmt>` | Enum | `json` | Output format: `json` or `text` |
| `--lang <tag>` | String | (auto) | Language for messages: `en`, `ja`, etc. |
| `--no-color` | Bool | `false` | Disable colored output |
| `--debug` | Bool | `false` | Enable debug logging (tokens masked) |
| `--allow-write` | Bool | `false` | Enable write operations |

## Command Structure

```
slack-rs [GLOBAL_FLAGS] <COMMAND> [SUBCOMMAND] [OPTIONS]
```

## Commands

### `auth` - Authentication Management

#### `auth login`
Authenticate with a Slack workspace via OAuth.

**Usage:**
```bash
slack-rs auth login --profile <name>
```

**Options:**
- `--profile <name>` (required): Profile name to create/update

**Behavior:**
1. Check if profile already exists
2. Start OAuth flow (PKCE + localhost callback)
3. Open browser for user authorization
4. Exchange code for token
5. Store token in file storage
6. Save profile metadata to `profiles.json`

**Output (JSON):**
```json
{
  "ok": true,
  "profile_name": "acme-work",
  "team_id": "T123ABC",
  "team_name": "Acme Corp",
  "user_id": "U456DEF",
  "scopes": ["search:read", "channels:read", "chat:write"]
}
```

---

#### `auth status`
Show authentication status for a profile.

**Usage:**
```bash
slack-rs auth status --profile <name>
```

**Output (JSON):**
```json
{
  "profile_name": "acme-work",
  "team_id": "T123ABC",
  "team_name": "Acme Corp",
  "user_id": "U456DEF",
  "user_name": "john.doe",
  "scopes": ["search:read", "channels:read", "chat:write"],
  "created_at": "2026-02-03T10:30:00Z",
  "last_used_at": "2026-02-03T15:45:00Z",
  "token_valid": true
}
```

---

#### `auth list`
List all configured profiles.

**Usage:**
```bash
slack-rs auth list
```

**Output (JSON):**
```json
{
  "profiles": [
    {
      "profile_name": "acme-work",
      "team_name": "Acme Corp",
      "team_id": "T123ABC",
      "user_id": "U456DEF",
      "last_used_at": "2026-02-03T15:45:00Z"
    },
    {
      "profile_name": "partner-ws",
      "team_name": "Partner Inc",
      "team_id": "T789GHI",
      "user_id": "U012JKL",
      "last_used_at": "2026-02-02T09:15:00Z"
    }
  ]
}
```

---

#### `auth rename`
Rename a profile.

**Usage:**
```bash
slack-rs auth rename --profile <old> --to <new>
```

**Options:**
- `--profile <old>` (required): Current profile name
- `--to <new>` (required): New profile name

**Behavior:**
- Updates `profile_name` in `profiles.json`
- Does not affect file storage entry (keyed by `team_id:user_id`)

---

#### `auth logout`
Remove authentication for a profile.

**Usage:**
```bash
slack-rs auth logout --profile <name>
```

**Behavior:**
1. Delete token from file storage
2. Remove profile from `profiles.json`

---

### `api` - Generic API Access

#### `api call`
Call any Slack Web API method.

**Usage:**
```bash
slack-rs --profile <name> api call <method> [key=value...] [--json '{...}']
```

**Arguments:**
- `<method>`: Slack API method (e.g., `search.messages`, `conversations.history`)
- `[key=value...]`: Form parameters (e.g., `channel=C123 limit=50`)

**Options:**
- `--json <json>`: Send JSON body instead of form parameters
- `--get`: Use GET instead of POST (default: POST)

**Examples:**
```bash
# Form parameters (default)
slack-rs --profile acme api call search.messages query="invoice" count=20

# JSON body
slack-rs --profile acme api call chat.postMessage --json '{
  "channel": "C123",
  "text": "Hello"
}'

# GET request
slack-rs --profile acme api call users.info --get user=U456
```

**Output:**
Raw Slack API response wrapped with metadata:
```json
{
  "meta": {
    "profile_name": "acme-work",
    "team_id": "T123ABC",
    "team_name": "Acme Corp",
    "user_id": "U456DEF",
    "method": "search.messages"
  },
  "response": {
    "ok": true,
    "messages": { ... }
  }
}
```

---

### `search` - Search Messages

**Usage:**
```bash
slack-rs --profile <name> search <query> [OPTIONS]
```

**Arguments:**
- `<query>`: Search query (Slack search syntax)

**Options:**
- `--limit <n>`: Maximum results (default: 20)
- `--sort <field>`: Sort by `timestamp` or `score` (default: `score`)
- `--order <dir>`: Sort order `asc` or `desc` (default: `desc`)

**Example:**
```bash
slack-rs --profile acme search "invoice in:#finance" --limit 50 --sort timestamp
```

---

### `conv` - Conversations

#### `conv list`
List conversations (channels, DMs, etc.).

**Usage:**
```bash
slack-rs --profile <name> conv list [OPTIONS]
```

**Options:**
- `--types <types>`: Comma-separated types: `public_channel`, `private_channel`, `im`, `mpim` (default: all)
- `--limit <n>`: Maximum results (default: 100)

---

#### `conv history`
Fetch conversation history.

**Usage:**
```bash
slack-rs --profile <name> conv history --channel <id> [OPTIONS]
```

**Options:**
- `--channel <id>` (required): Channel ID
- `--oldest <ts>`: Oldest timestamp (inclusive)
- `--latest <ts>`: Latest timestamp (exclusive)
- `--limit <n>`: Maximum messages (default: 100)

---

### `users` - User Information

#### `users info`
Get user information.

**Usage:**
```bash
slack-rs --profile <name> users info --user <id>
```

**Options:**
- `--user <id>` (required): User ID

---

### `msg` - Message Operations

**All `msg` commands require `--allow-write` flag.**

#### `msg post`
Post a new message.

**Usage:**
```bash
slack-rs --profile <name> --allow-write msg post --channel <id> --text <text> [OPTIONS]
```

**Options:**
- `--channel <id>` (required): Channel ID
- `--text <text>` (required): Message text
- `--thread-ts <ts>`: Reply to thread

---

#### `msg update`
Update an existing message.

**Usage:**
```bash
slack-rs --profile <name> --allow-write msg update --channel <id> --ts <ts> --text <text>
```

**Options:**
- `--channel <id>` (required): Channel ID
- `--ts <ts>` (required): Message timestamp
- `--text <text>` (required): New message text

---

#### `msg delete`
Delete a message (destructive operation).

**Usage:**
```bash
slack-rs --profile <name> --allow-write msg delete --channel <id> --ts <ts> [--yes]
```

**Options:**
- `--channel <id>` (required): Channel ID
- `--ts <ts>` (required): Message timestamp
- `--yes`: Skip confirmation prompt

**Behavior:**
- Without `--yes`: Display confirmation prompt
- With `--yes`: Delete immediately

---

### `react` - Reactions

**All `react` commands require `--allow-write` flag.**

#### `react add`
Add a reaction to a message.

**Usage:**
```bash
slack-rs --profile <name> --allow-write react add --channel <id> --ts <ts> --emoji <emoji>
```

**Options:**
- `--channel <id>` (required): Channel ID
- `--ts <ts>` (required): Message timestamp
- `--emoji <emoji>` (required): Emoji name (e.g., `:thumbsup:`)

---

#### `react remove`
Remove a reaction from a message.

**Usage:**
```bash
slack-rs --profile <name> --allow-write react remove --channel <id> --ts <ts> --emoji <emoji>
```

**Options:**
- `--channel <id>` (required): Channel ID
- `--ts <ts>` (required): Message timestamp
- `--emoji <emoji>` (required): Emoji name

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid arguments, API error, etc.) |
| 2 | Authentication error (missing/invalid token) |
| 3 | Permission error (missing scope, write not allowed) |
| 4 | Rate limit exceeded (after retries) |

## Output Formats

### JSON (default)
- Machine-readable
- Always includes `meta` block with profile/team/user context
- Suitable for piping to `jq`, scripts, etc.

### Text
- Human-readable summary
- Shows key information only
- Always includes workspace identifier
- Example:
  ```
  [acme-work / Acme Corp] Message posted successfully
  Channel: #general (C123ABC)
  Timestamp: 1234567890.123456
  ```
