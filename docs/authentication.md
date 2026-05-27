# Authentication Guide

## Overview

`slack-rs` uses OAuth 2.0 with PKCE for Slack workspace authentication. This guide covers all authentication workflows in detail.

For implementation details of the OAuth protocol, see [docs/oauth.md](oauth.md).

## Quick Setup: App Manifest Flow (Recommended)

The simplest way to authenticate is using `--cloudflared`, which handles tunnel + manifest generation automatically.

**Prerequisites:**
- Install [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation/)

**Steps:**

```bash
# 1. Start login — manifest YAML is generated automatically
slack-rs auth login my-workspace --cloudflared
# → Tunnel starts, manifest YAML saved to ~/.config/slack-rs/<profile>_manifest.yml

# 2. Create Slack App from manifest
#    Go to https://api.slack.com/apps → "Create New App" → "From an app manifest"
#    Paste the generated YAML, click "Create"

# 3. Enter credentials when prompted
#    Copy Client ID and Client Secret from "Basic Information" → "App Credentials"

# 4. OAuth flow completes automatically
#    Browser opens → Click "Allow" → Token saved
```

**Customizing scopes:**

```bash
slack-rs auth login my-workspace --cloudflared --bot-scopes chat:write --user-scopes users:read,channels:read
```

Common scopes: `chat:write`, `users:read`, `channels:read`, `files:read`, `search:read`, `reactions:write`. See [full list](https://api.slack.com/scopes).

## Manual Setup

### 1. Create a Slack App

1. Go to https://api.slack.com/apps → "Create New App" → "From scratch"
2. Name your app, select a development workspace
3. Under "OAuth & Permissions", add redirect URL: `http://127.0.0.1:8765/callback`
4. Add required OAuth scopes under "User Token Scopes"
5. Copy **Client ID** and **Client Secret** from "Basic Information" → "App Credentials"

### 2. Save OAuth Config

```bash
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "chat:write,users:read,channels:read"
```

### 3. Authenticate

```bash
slack-rs auth login my-workspace
# Browser opens → Click "Allow" → Token saved
```

### Alternative: Provide Credentials at Login

```bash
# Client ID as argument, secret prompted
slack-rs auth login my-workspace --client-id 123456789012.1234567890123

# Fully interactive (both ID and secret prompted)
slack-rs auth login my-workspace
```

## Using Tunnels for Remote Authentication

When `localhost` is not accessible (SSH, Docker, cloud instances):

### Method A: Built-in Cloudflare Tunnel (Recommended)

```bash
slack-rs auth login my-workspace --cloudflared
```

The CLI automatically starts the tunnel, generates a manifest with the correct redirect URL, and handles the OAuth callback. The tunnel is closed after authentication.

### Method B: Manual Tunnel

```bash
# Start tunnel
cloudflared tunnel --url http://localhost:8765

# Configure redirect URI with the tunnel URL
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri https://xyz-def-ghi.trycloudflare.com/callback \
  --scopes "chat:write,users:read"

slack-rs auth login my-workspace
```

**Security notes:**
- Tunnel URLs are temporary and change each restart
- Anyone with the tunnel URL can access your callback endpoint during auth
- Built-in tunnel support auto-closes the tunnel after authentication

## Auth Commands Reference

### Login

```bash
slack-rs auth login [profile-name]              # Use saved config or prompts
slack-rs auth login [profile-name] --client-id <id>  # Explicit client ID
slack-rs auth login [profile-name] --cloudflared     # Cloudflare Tunnel mode
```

### Status

```bash
slack-rs auth status [profile-name]  # Check auth and profile info
```

### List

```bash
slack-rs auth list  # Show all saved profiles
```

### Rename

```bash
slack-rs auth rename <old-name> <new-name>
```

### Logout

```bash
slack-rs auth logout <profile-name>  # Remove profile and credentials
```

## OAuth Configuration Management

### Set

```bash
slack-rs config oauth set <profile> \
  --client-id <id> \
  --redirect-uri <uri> \
  --scopes <scopes>
# Use --scopes "all" for comprehensive preset
```

### Show

```bash
slack-rs config oauth show <profile>
```

### Delete

```bash
slack-rs config oauth delete <profile>
```

## Profile Export/Import

Create encrypted backups or migrate profiles between machines.

### Export

```bash
slack-rs auth export --profile <name> --out <file> --passphrase-prompt
slack-rs auth export --all --out <file> --passphrase-prompt
```

### Import

```bash
slack-rs auth import --profile <name> --in <file> --passphrase-prompt
slack-rs auth import --all --in <file> --passphrase-prompt
```

### Security Details

- **Encryption**: AES-256-GCM
- **Key Derivation**: Argon2id with random salt
- **File Permissions**: `0600` (owner read/write only)

### Best Practices

- Use strong passphrases (16+ characters)
- Store exported files in secure locations
- Never commit `*.enc` files to version control
- Delete old exports after successful import

## Security Model

### Credential Storage

| Credential | Storage | Location |
|-----------|---------|----------|
| Access Token | File-based | `~/.config/slack-rs/tokens.json` (0600) |
| Client ID | Plain JSON | `~/.config/slack-rs/profiles.json` |
| Client Secret | File-based | `~/.config/slack-rs/tokens.json` (0600) |

- Client IDs are not considered sensitive (OAuth 2.0 spec)
- Access tokens and secrets are never logged or printed

### Write Protection

Set `SLACKCLI_ALLOW_WRITE=false` to prevent accidental write operations:

```bash
export SLACKCLI_ALLOW_WRITE=false
slack-rs msg post C123 "Hello"  # → Error: Write operation denied
```

For more details, see [docs/security.md](security.md) and [docs/config-and-storage.md](config-and-storage.md).
