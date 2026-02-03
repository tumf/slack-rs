# OAuth Implementation Specification

## Overview
This CLI uses OAuth 2.0 with PKCE (Proof Key for Code Exchange) to obtain user tokens from Slack.

## OAuth Flow

### 1. Initiate Login
User runs: `slackcli auth login --profile myworkspace`

### 2. Generate PKCE Parameters
- **code_verifier**: Random 128-character string (base64url-encoded)
- **code_challenge**: SHA256 hash of code_verifier (base64url-encoded)
- **state**: Random 32-character string (CSRF protection)

### 3. Start Localhost Callback Server
- Bind to `127.0.0.1` on an ephemeral port (OS-assigned)
- Store `state` in server context for validation
- Endpoint: `GET /auth/callback`

### 4. Build Authorization URL
```
https://slack.com/oauth/v2/authorize?
  client_id={SLACKCLI_CLIENT_ID}
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
  "client_id": "{SLACKCLI_CLIENT_ID}",
  "client_secret": "{SLACKCLI_CLIENT_SECRET}",
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
- `SLACKCLI_CLIENT_ID`: Slack OAuth client ID
- `SLACKCLI_CLIENT_SECRET`: Slack OAuth client secret

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
- Error: `SLACKCLI_CLIENT_ID not set. Please configure OAuth credentials.`
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
- Use `oauth2` crate for PKCE generation and token exchange
- Use `axum` or `tiny_http` for callback server
- Use `webbrowser` crate for cross-platform browser opening
- Timeout callback server after 5 minutes (configurable)
