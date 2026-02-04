# OAuth Implementation

## Overview
This CLI uses OAuth 2.0 with PKCE (Proof Key for Code Exchange) to obtain user tokens from Slack. This document describes the implemented OAuth flow.

## OAuth Flow

### 1. Initiate Login
User runs: `slackcli auth login --profile myworkspace`

### 2. Generate PKCE Parameters
- **code_verifier**: Random 128-character string (base64url-encoded)
- **code_challenge**: SHA256 hash of code_verifier (base64url-encoded)
- **state**: Random 32-character string (CSRF protection)

### 3. Start Localhost Callback Server
- Bind to `127.0.0.1:3000` (fixed port)
- Store `state` in server context for validation
- Accept any GET request with query parameters

### 4. Build Authorization URL
```
https://slack.com/oauth/v2/authorize?
  client_id={SLACKRS_CLIENT_ID}
  &scope={scopes}
  &redirect_uri=http://127.0.0.1:{port}/auth/callback
  &state={state}
  &code_challenge={code_challenge}
  &code_challenge_method=S256
```

### 5. Open Browser
- Use `open` (macOS), `xdg-open` (Linux), or `start` (Windows)
- Display message: "Opening browser for authentication. Waiting for callback..."
- If browser fails to open: print URL and instruct user to open manually

### 6. Handle Callback
- Slack redirects to: `http://127.0.0.1:{port}/auth/callback?code={code}&state={state}`
- Validate `state` matches stored value (reject if mismatch)
- Extract `code` parameter
- Respond to browser with success page (HTML)
- Shutdown callback server

### 7. Exchange Code for Token
POST to `https://slack.com/api/oauth.v2.access`:
```json
{
  "client_id": "{SLACKRS_CLIENT_ID}",
  "client_secret": "{SLACKRS_CLIENT_SECRET}",
  "code": "{code}",
  "redirect_uri": "http://127.0.0.1:{port}/auth/callback",
  "code_verifier": "{code_verifier}"
}
```

### 8. Process Response
Expected response:
```json
{
  "ok": true,
  "access_token": "xoxp-...",
  "token_type": "user",
  "scope": "search:read,channels:read,...",
  "authed_user": {
    "id": "U456DEF"
  },
  "team": {
    "id": "T123ABC",
    "name": "Acme Corp"
  }
}
```

### 9. Store Profile and Token
- Extract: `team_id`, `team_name`, `user_id`, `scopes`
- Check if `(team_id, user_id)` already exists in `profiles.json`
  - If exists: update token and metadata
  - If new: create new profile entry
- Store token in keyring with key `{team_id}:{user_id}`
- Save updated `profiles.json`

## Required Environment Variables
- `SLACKRS_CLIENT_ID`: Slack OAuth client ID
- `SLACKRS_CLIENT_SECRET`: Slack OAuth client secret

## Recommended Scopes

### Read-Only Set (Minimal)
```
search:read
channels:read
groups:read
im:read
mpim:read
channels:history
groups:history
im:history
mpim:history
users:read
```

### Write Set (Additional)
```
chat:write
reactions:write
```

### Admin/Enterprise (Optional)
```
admin.conversations:read
admin.users:read
```

## Error Handling

### Missing Environment Variables
- Error: `SLACKRS_CLIENT_ID not set. Please configure OAuth credentials.`
- Exit code: 1

### State Mismatch (CSRF)
- Error: `OAuth state mismatch. Possible CSRF attack. Aborting.`
- Exit code: 1

### Slack API Error
- Error: `OAuth failed: {error_description}`
- Exit code: 1

### Port Binding Failure
- Try multiple ports (e.g., 8000-8010)
- If all fail: Error: `Failed to bind callback server. Ensure ports 8000-8010 are available.`
- Exit code: 1

## Security Considerations
- **PKCE**: Protects against authorization code interception
- **State**: Protects against CSRF attacks
- **Localhost binding**: Reduces attack surface (only local connections accepted)
- **Ephemeral server**: Callback server shuts down immediately after receiving code
- **Token storage**: Never log or print tokens; store only in keyring

## Implementation Notes
- Custom PKCE implementation using `sha2` and `base64` crates
- Token exchange using `reqwest` HTTP client
- Custom callback server using `tokio::net::TcpListener`
- Browser opening via platform-specific commands (`open`, `xdg-open`, `cmd /C start`)
- Callback server times out after 5 minutes (300 seconds, configurable)

## Module Structure

- **oauth/mod.rs**: Main OAuth coordination and token exchange
- **oauth/pkce.rs**: PKCE code verifier/challenge generation
- **oauth/types.rs**: OAuth types, configuration, and error handling
- **oauth/server.rs**: Local callback server implementation
- **auth/commands.rs**: CLI command implementations (login, status, list, rename, logout)

## Testing

The implementation includes comprehensive unit and integration tests:

- Unit tests for PKCE generation and validation
- Unit tests for OAuth configuration validation
- Unit tests for callback server query string parsing
- Integration tests with mock OAuth server using `wiremock`
- Integration tests for auth commands with profile and token storage

Run tests with:
```bash
cargo test
cargo test --test oauth_integration
cargo test --test auth_integration
```
