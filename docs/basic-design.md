# Slack CLI (Rust) - Basic Design

## Goals
- Provide a pipe-friendly Slack Web API CLI with JSON-first output.
- Support multiple accounts as a first-class concept:
  - Multiple workspaces
  - Multiple users within the same workspace
- Keep the API surface "complete" via a generic `api call` command, plus thin human-friendly wrappers.

## Non-Goals (initial)
- RTM / Socket Mode event subscriptions
- Enterprise Grid admin/audit APIs (may be added later)

## Terminology
- **Account (auth context)**: a unique Slack identity defined by `(team_id, authed_user_id)`.
- **Profile**: a human-friendly alias that points to exactly one Account.
- **Token**: Slack user token obtained via OAuth (`oauth.v2.access`).

## High-Level Architecture
- **CLI layer (clap)**:
  - Parses global flags and subcommands.
  - Enforces safety gates for write operations.
- **Config layer**:
  - Stores profile metadata (non-secret).
  - Resolves `--profile <name>` -> `(team_id, user_id)` key.
- **Token store layer (keyring)**:
  - Stores secrets only (access token, optionally refresh token) keyed by stable account id.
- **Slack API layer (reqwest)**:
  - Sends requests to `https://slack.com/api/{method}`.
  - Implements retry/backoff and rate-limit handling.
  - Masks tokens in logs.
- **i18n layer**:
  - Localizes human-facing messages (errors, prompts, auth instructions).
  - JSON output remains language-independent.

## Account + Profile Model (Core)

### Stable Account Key
- **Canonical key**: `{team_id}:{user_id}` (example: `T123:U456`)
- **Rationale**:
  - Uniquely identifies an authenticated user within a workspace.
  - Allows multiple users in the same workspace.
  - Profile renames do not affect keyring entries.

### Profile Rules
- `--profile` is **required** for all commands that hit Slack APIs.
- A profile maps to exactly one stable account key.
- On `auth login`, if the same stable account key already exists:
  - Replace stored token (re-login)
  - Update metadata (scopes, team/user names)
  - Keep the existing profile name unless the user explicitly chooses/renames

## Storage

### Config (non-secret)
- **File**: `profiles.json` under an OS-appropriate config directory (via `directories` crate).
- **Contains**:
  - `profile_name`
  - `team_id`, `team_name`
  - `user_id`, optional `user_name`
  - `scopes`
  - `created_at`, `last_used_at`

### Token Store (secret)
- **Keyring**:
  - service: `slackcli`
  - username: `{team_id}:{user_id}`
  - secret: token payload (either raw access token or JSON blob)

## OAuth (PKCE + localhost callback)
- OAuth uses PKCE + state to mitigate interception and CSRF.
- Callback server binds to loopback (recommended: `127.0.0.1`) and an ephemeral free port.
- **Required runtime configuration** (do not hardcode):
  - `SLACKRS_CLIENT_ID`
  - `SLACKRS_CLIENT_SECRET`
- **Redirect URL pattern**:
  - `http://127.0.0.1:{port}/auth/callback`

## Commands (Overview)

### Auth
- `auth login --profile <name>`
- `auth status --profile <name>`
- `auth list`
- `auth rename --profile <old> --to <new>`
- `auth logout --profile <name>`

### API
- `api call <method> [key=value...] [--json '{...}']`

### Wrappers
- `search`
- `conv list/history`
- `users info`
- `msg post/update/delete`
- `react add/remove`

## Safety Gates (Write Operations)
- Write operations require global flag `--allow-write`.
- Destructive operations additionally require `--yes`.
  - Initially: `msg delete` is destructive
- If gates are not satisfied:
  - Exit non-zero with a localized error message.

## Output Contract
- **Default output format**: JSON.
- Always include an execution context block in JSON responses:
  - `meta.profile_name`
  - `meta.team_id`, `meta.team_name`
  - `meta.user_id` (and `meta.user_name` if known)
- Text output is for summaries only and must still show team/user identifiers.

## Rate Limiting / Retries
- Respect HTTP 429 and `Retry-After`.
- Exponential backoff with jitter for transient failures.
- Bound retries to avoid infinite loops; surface final errors clearly.

## Security
- Never print tokens.
- Mask Authorization headers in debug logs.
- Keep secrets only in keyring; config file must not contain tokens.

## Implementation Phases

### Phase 1: Core Infrastructure
1. Profile/config management (`(team_id, user_id)` key design)
2. Keyring integration
3. OAuth login flow (PKCE + localhost callback)
4. `auth` commands (login/status/list/logout/rename)

### Phase 2: API Foundation
5. Generic `api call` with retry/rate-limit handling
6. Response metadata injection (profile/team/user context)
7. Token masking in logs

### Phase 3: Wrappers
8. Read-only wrappers (`search`, `conv`, `users`)
9. Write wrappers with `--allow-write` gate (`msg`, `react`)
10. Destructive operation confirmation (`--yes` for `msg delete`)

### Phase 4: Polish
11. i18n (en/ja) for human-facing messages
12. Text output formatting
13. Error message improvements
