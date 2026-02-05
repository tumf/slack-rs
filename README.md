# slack-rs

A Slack CLI tool written in Rust that provides comprehensive access to the Slack Web API using OAuth authentication.

[![CI](https://github.com/tumf/slack-rs/workflows/CI/badge.svg)](https://github.com/tumf/slack-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/slack-rs.svg)](https://crates.io/crates/slack-rs)
[![Documentation](https://docs.rs/slack-rs/badge.svg)](https://docs.rs/slack-rs)
[![Downloads](https://img.shields.io/crates/d/slack-rs.svg)](https://crates.io/crates/slack-rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

`slack-rs` is a command-line tool designed for interacting with Slack workspaces using your personal OAuth credentials. It supports multiple workspace profiles, secure token storage, and provides both generic API access and convenient wrapper commands for common operations.

### Key Features

- 🔐 **OAuth Authentication** with PKCE flow
- 🏢 **Multiple Workspace Support** via profiles
- 🔒 **Secure Token Storage** using OS keyring (Keychain/Secret Service)
- 🔄 **Profile Import/Export** with encryption
- 📡 **Generic API Access** - call any Slack Web API method
- 🛠️ **Wrapper Commands** for common operations
- 🔁 **Smart Retry Logic** with exponential backoff and rate limit handling

## TL;DR

```bash
# Install from crates.io
cargo install slack-rs

# Or build from source
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
make build
make test
```

## Installation

### Prerequisites

- Rust 1.70+ (tested with 1.92.0)
- A Slack app with OAuth credentials ([create one here](https://api.slack.com/apps))

### Install from crates.io (Recommended)

```bash
cargo install slack-rs
```

The binary will be installed to `~/.cargo/bin/slack-rs` (ensure `~/.cargo/bin` is in your PATH).

### Build from Source

```bash
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
cargo build --release
```

The binary will be available at `target/release/slack-rs`.

### Install from Local Source

```bash
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
cargo install --path .
```

## Quick Start

### 1. Set Up OAuth Credentials

#### Quick Setup: Using App Manifest (Recommended)

The most recommended login flow is to install **cloudflared** and use `--cloudflared`.
In this mode, `slack-rs auth login` generates a Slack App Manifest for you (and copies it to your clipboard).

The intended flow is:

1. **Create a Slack app and get credentials**:
   - Go to https://api.slack.com/apps
   - Click "Create New App" ("From scratch" is fine)
   - In "Basic Information" → "App Credentials", copy your **Client ID** and **Client Secret**
2. **Install cloudflared**:
   - Follow Cloudflare docs: https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation/
3. **Start login with --cloudflared (manifest is generated automatically)**:
   ```bash
    slack-rs auth login my-workspace --client-id 123456789012.1234567890123 --cloudflared
    # You'll be prompted for the client secret (hidden)
    # A manifest YAML is generated, saved, and copied to clipboard
    ```
4. **Paste the generated YAML into Slack**:
   - In your Slack app settings, open "App Manifest"
   - Paste the generated YAML (from clipboard or `~/.config/slack-rs/<profile>_manifest.yml`)
   - Apply the changes
5. **Return to the terminal and press Enter**:
   - The CLI opens your browser
   - Click "Allow"
   - The CLI exchanges the code for tokens and saves them securely

**💡 Manifest Benefits:**
- ✅ Redirect URL and scopes are configured for you
- ✅ Less manual Slack UI configuration
- ✅ Manifest is saved to `~/.config/slack-rs/<profile>_manifest.yml` and copied to clipboard (best effort)
- ✅ Easy to reproduce and share

**Customizing Scopes:**

If you need different scopes, change the scopes you pass to `auth login` (or edit the manifest YAML before pasting it into Slack).

Flags:
- `--bot-scopes <scopes>`: comma-separated list or `all`
- `--user-scopes <scopes>`: comma-separated list or `all`

Common scopes:
- `chat:write` - Post messages
- `users:read` - View users
- `channels:read` - List public channels
- `files:read` - Access files
- `search:read` - Search workspace content
- `reactions:write` - Add/remove reactions
- See full list: https://api.slack.com/scopes

#### Manual Setup (Alternative)

If you prefer manual configuration:

1. **Navigate to Slack API**: Go to https://api.slack.com/apps
2. **Create an app**:
   - Click "Create New App"
   - Choose "From scratch"
   - Name your app (e.g., "My Slack CLI")
   - Select a development workspace
3. **Configure OAuth & Permissions**:
   - In the left sidebar, click "OAuth & Permissions"
   - Scroll to "Redirect URLs" section
   - Add redirect URL: `http://127.0.0.1:8765/callback`
   - Click "Save URLs"
4. **Add OAuth Scopes**:
   - Scroll to "Scopes" section under "User Token Scopes"
   - Add required scopes for your use case:
     - `chat:write` - Post messages
     - `users:read` - View users
     - `channels:read` - List public channels
     - `search:read` - Search workspace content
     - Add more as needed based on API methods you'll use
5. **Get your credentials**:
   - Go to "Basic Information" → "App Credentials"
   - Copy your **Client ID** (looks like `123456789012.1234567890123`)
   - Click "Show" and copy your **Client Secret** (looks like `abcdef1234567890abcdef1234567890`)

#### Providing Credentials

**Option A: Save credentials to profile (recommended for most users):**

```bash
# Save OAuth config to profile (will be prompted for client secret)
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "chat:write,users:read,channels:read"

# Then authenticate using saved config
slack-rs auth login my-workspace
```

**Option B: Provide during login (quick one-time use):**

```bash
# Provide client ID as argument, secret will be prompted securely
slack-rs auth login my-workspace --client-id 123456789012.1234567890123
```

**Option C: Interactive prompts:**

```bash
# Tool will prompt for both client ID and secret
slack-rs auth login my-workspace
# Enter OAuth client ID: [type your client ID]
# Enter OAuth client secret: [type your secret - hidden]
```

**💡 Pro Tip**: Use Option A for persistent configuration.

### 2. Authenticate

**Login to your Slack workspace:**

```bash
# Method 1: Using saved OAuth config (recommended)
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "chat:write,users:read,channels:read"
slack-rs auth login my-workspace

# Method 2: Provide client ID during login
slack-rs auth login my-workspace --client-id 123456789012.1234567890123

# Method 3: Interactive prompts
slack-rs auth login my-workspace
```

**What happens during login:**

1. **Credentials collected**: Client ID and secret are obtained (from saved profile/keyring, CLI args, or prompts)

When using `--cloudflared`:
2. **Tunnel started**: `cloudflared` tunnel is started and a public redirect URL is determined
3. **Manifest generated**: A Slack App Manifest YAML is generated, saved, and copied to clipboard (best effort)
4. **You paste the manifest in Slack**: Configure your Slack app using the generated manifest
5. **Press Enter to continue**: The CLI starts the OAuth flow

OAuth flow:
6. **Browser opens**: OAuth authorization page opens automatically
7. **User authorization**: Click "Allow" to grant permissions to your app
8. **Callback handled**: Callback server receives OAuth callback with authorization code
9. **Token exchange**: Code is exchanged for access token
10. **Secure storage**: Profile and token are saved securely
   - Profile metadata → `~/.config/slack-rs/profiles.json`
   - Access token → OS Keyring (Keychain/Secret Service/Credential Manager)

**After successful authentication:**

```
✓ Authentication successful!
Profile 'my-workspace' saved.
```

**Per-Profile OAuth Settings:**
- ✅ Each profile can store its own OAuth client ID, redirect URI, and scopes
- 💾 OAuth config saved in `~/.config/slack-rs/profiles.json`
- 🔒 Client secret saved securely in OS keyring (prompted only if missing)
- 🔄 Subsequent logins reuse saved configuration automatically

#### Using Tunneling Services for Remote Authentication

When authenticating from a remote server or environment where `localhost` is not accessible (e.g., SSH, Docker, cloud instances), the most recommended flow is:

1. Install **cloudflared**
2. Run `slack-rs auth login ... --cloudflared`

This avoids manually starting a tunnel or managing redirect URLs.

**Method A: Built-in Cloudflare Tunnel Support (Easiest)**

`slack-rs` also supports Cloudflare Tunnel with automatic management:

1. **Install cloudflared**: Download from [Cloudflare](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation/)

2. **Configure Slack App with cloudflared redirect URL**:
   - Go to https://api.slack.com/apps → Your App → OAuth & Permissions
   - Add redirect URL: `https://your-tunnel.trycloudflare.com/callback` (you'll get the exact URL from cloudflared)
   - Click "Save URLs"

3. **Authenticate with --cloudflared flag**:
    ```bash
    # The tool will automatically start cloudflared and handle the tunnel
    slack-rs auth login my-workspace --cloudflared
   
   # Or with client ID
    slack-rs auth login my-workspace --client-id 123456789012.1234567890123 --cloudflared
    ```

With `--cloudflared`, you do not need to manually start a tunnel or copy/paste a tunnel URL into your config.
The CLI starts the tunnel, generates a manifest containing the correct redirect URL, and copies it to your clipboard.
You only need to paste the manifest into Slack and press Enter to continue.

The `--cloudflared` flag automatically:
- Starts a Cloudflare Tunnel on port 8765
- Displays the public URL for your redirect URI
- Handles the OAuth callback through the tunnel
- Closes the tunnel after authentication

**Method B: Manual Tunnel Setup (Advanced)**

If you prefer to manage the tunnel yourself:

1. **Start cloudflared manually**:
   ```bash
   # cloudflared
   cloudflared tunnel --url http://localhost:8765
   ```

2. **Configure Slack App with the tunnel URL**:
   - Add the tunnel URL as redirect URI (e.g., `https://xyz-def-ghi.trycloudflare.com/callback`)

3. **Authenticate with custom redirect URI**:
    ```bash
     slack-rs config oauth set my-workspace \
       --client-id 123456789012.1234567890123 \
       --redirect-uri https://xyz-def-ghi.trycloudflare.com/callback \
       --scopes "chat:write,users:read"
     slack-rs auth login my-workspace
    ```

If you are using the manifest-based flow described above, the manual `config oauth set --redirect-uri ...` step is usually unnecessary.
Using `--cloudflared` avoids having to manually manage the tunnel URL.

**Security Notes:**
- ⚠️ Tunnel URLs are temporary and change each time you restart the service
- ⚠️ Anyone with the tunnel URL can access your callback endpoint during authentication
- ✅ The built-in tunnel support automatically closes the tunnel after authentication
- ✅ Tunnels are only active during the authentication process

#### ngrok status

The `--ngrok` flag exists in the CLI help, but ngrok tunnel automation is not implemented in this version.

### 3. Make API Calls

**Generic API call:**

```bash
slack-rs api call chat.postMessage channel=C123456 text="Hello from slack-rs!"
```

**Check authentication status:**

```bash
slack-rs auth status my-workspace
```

**View saved OAuth configuration:**

```bash
slack-rs config oauth show my-workspace
```

**List all profiles:**

```bash
slack-rs auth list
```

## Usage

### Authentication Commands

#### Login

Authenticate with a Slack workspace and save credentials.

```bash
# Basic login (uses saved profile or prompts)
slack-rs auth login [profile-name]

# Login with specific client ID
slack-rs auth login [profile-name] --client-id <client-id>

# Login with Cloudflare Tunnel (for remote/SSH environments)
slack-rs auth login [profile-name] --cloudflared

# Examples:
slack-rs auth login                           # Profile named "default"
slack-rs auth login my-team                   # Profile named "my-team"
slack-rs auth login dev --client-id 12345.67  # With explicit client ID
slack-rs auth login server --cloudflared      # Using Cloudflare Tunnel
```

**Flags:**
- `--client-id <id>`: Specify OAuth client ID
- `--cloudflared`: Automatically start Cloudflare Tunnel for remote authentication

Note: `--ngrok` exists in the CLI help, but ngrok tunnel automation is not implemented in v0.1.6.

#### Status

Check authentication status and profile information.

```bash
# Check specific profile
slack-rs auth status <profile-name>

# Check default profile
slack-rs auth status

# Example output:
# Profile: my-workspace
# Team: My Team (T123456)
# User: John Doe (U789012)
# Client ID: 123456789012.123456789012
# Status: ✓ Authenticated
```

#### List

Display all saved profiles.

```bash
slack-rs auth list

# Example output:
# Profiles:
#   • default (My Team / john.doe)
#   • dev-workspace (Dev Team / jane.smith)
#   • prod-workspace (Prod Team / jane.smith)
```

#### Rename

Rename an existing profile.

```bash
slack-rs auth rename <old-name> <new-name>

# Example:
slack-rs auth rename default my-main-workspace
```

#### Logout

Remove profile and delete all associated credentials.

```bash
slack-rs auth logout <profile-name>

# Example:
slack-rs auth logout old-workspace
# ✓ Profile 'old-workspace' removed
# ✓ Credentials deleted from keyring
```

#### Export

Create encrypted backup of profiles.

```bash
# Export single profile
slack-rs auth export --profile <name> --out <file> [--passphrase-prompt]
slack-rs auth export --profile <name> --out <file> [--yes]

# Export all profiles
slack-rs auth export --all --out <file> [--passphrase-prompt]
slack-rs auth export --all --out <file> [--yes]

# Examples:
slack-rs auth export --profile prod --out prod-backup.enc --passphrase-prompt
slack-rs auth export --all --out all-profiles-$(date +%Y%m%d).enc --passphrase-prompt

# With environment variable (for automation)
export SLACKRS_KEYRING_PASSWORD="strong-passphrase"
slack-rs auth export --profile prod --out backup.enc --yes
```

**Flags:**
- `--profile <name>`: Export specific profile
- `--all`: Export all profiles
- `--out <file>`: Output file path
- `--passphrase-prompt`: Prompt for passphrase securely (recommended)
- `--yes`: Skip confirmation (use with `SLACKRS_KEYRING_PASSWORD` env var)

#### Import

Restore profiles from encrypted backup.

```bash
# Import single profile
slack-rs auth import --profile <name> --in <file> [--passphrase-prompt]

# Import all profiles
slack-rs auth import --all --in <file> [--passphrase-prompt]

# Examples:
slack-rs auth import --profile prod --in backup.enc --passphrase-prompt
slack-rs auth import --all --in all-profiles.enc --passphrase-prompt

# With environment variable
export SLACKRS_KEYRING_PASSWORD="strong-passphrase"
slack-rs auth import --all --in backup.enc
```

**Flags:**
- `--profile <name>`: Import specific profile
- `--all`: Import all profiles from file
- `--in <file>`: Input file path
- `--passphrase-prompt`: Prompt for passphrase securely (recommended)

### Configuration Commands

#### OAuth Configuration Management

Manage OAuth settings for each profile independently.

**Set OAuth configuration:**

```bash
slack-rs config oauth set <profile> \
  --client-id <client-id> \
  --redirect-uri <redirect-uri> \
  --scopes <scopes>

# Examples:
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "chat:write,users:read,channels:read"

# Use comprehensive scope preset
slack-rs config oauth set my-workspace \
  --client-id 123456789012.1234567890123 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "all"
```

**Show OAuth configuration:**

```bash
slack-rs config oauth show <profile>

# Example output:
# OAuth configuration for profile 'my-workspace':
#   Client ID: 123456789012.1234567890123
#   Redirect URI: http://127.0.0.1:8765/callback
#   Scopes: chat:write, users:read, channels:read
#   Client secret: (saved in keyring)
```

**Delete OAuth configuration:**

```bash
slack-rs config oauth delete <profile>

# Example:
slack-rs config oauth delete old-workspace
# ✓ OAuth configuration deleted for profile 'old-workspace'
```

#### Per-Profile OAuth Settings

Each profile can store its own OAuth configuration, enabling flexible multi-workspace and multi-app workflows.

**Benefits:**
- ✅ **Different Slack apps per workspace**: Use separate apps for different teams
- ✅ **Development/Production separation**: Test with dev app, deploy with prod app
- ✅ **Granular permission control**: Different scopes for different profiles
- ✅ **Persistent configuration**: Save OAuth settings once, reuse forever
- ✅ **Team collaboration**: Each team member can use their own Slack app
- ✅ **Easy switching**: No need to re-enter credentials when switching profiles

**How it works:**

| Step | Action | Storage Location |
|------|--------|------------------|
| 1️⃣ | Set OAuth config via `config oauth set` | `~/.config/slack-rs/profiles.json` + OS Keyring |
| 2️⃣ | Authenticate via `auth login` | Browser OAuth flow |
| 3️⃣ | Access token saved securely | OS Keyring |
| 4️⃣ | On re-login, saved config is reused | Auto-loaded from profile |

**Examples:**

```bash
# Scenario 1: Development workspace with dev app
slack-rs config oauth set dev-workspace \
  --client-id 111111111111.222222222222 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "chat:write,users:read"
slack-rs auth login dev-workspace

# Scenario 2: Production workspace with prod app and comprehensive scopes
slack-rs config oauth set prod-workspace \
  --client-id 333333333333.444444444444 \
  --redirect-uri http://127.0.0.1:8765/callback \
  --scopes "all"
slack-rs auth login prod-workspace

# Scenario 3: Re-authenticate dev-workspace (reuses saved config)
slack-rs auth login dev-workspace
# ℹ Using saved OAuth configuration
# [Browser opens automatically]

# Scenario 4: Check current OAuth configuration
slack-rs config oauth show dev-workspace
```

**Security Notes:**
- **Client IDs**: Saved in profile JSON (not sensitive per OAuth 2.0 spec)
- **Client Secrets**: Saved securely in OS keyring (Keychain/Secret Service/Credential Manager)
- **Access Tokens**: Always saved securely in OS keyring
- **Configuration Files**: Profile metadata stored in `~/.config/slack-rs/profiles.json` with 0600 permissions

### API Calls

**Generic API access:**

```bash
slack-rs api call <method> [key=value...]

# Examples:
slack-rs api call users.info user=U123456
slack-rs api call conversations.history channel=C123456 limit=50
slack-rs api call search.messages query="important" count=20
```

**Form-encoded arguments:**
```bash
slack-rs api call chat.postMessage channel=C123 text="Hello" thread_ts=1234567.123
```

### Output Format

All commands output JSON with a unified envelope structure that includes both the Slack API response and execution metadata.

**Default output (with envelope):**
```json
{
  "response": {
    "ok": true,
    "channels": [
      {"id": "C123", "name": "general"}
    ]
  },
  "meta": {
    "profile_name": "default",
    "team_id": "T123ABC",
    "user_id": "U456DEF",
    "method": "conversations.list",
    "command": "conv list"
  }
}
```

The `meta` object provides useful context:
- `profile_name`: Profile used for the request (null if not specified)
- `team_id`: Slack team/workspace ID
- `user_id`: User ID from the profile
- `method`: Slack API method called
- `command`: CLI command executed (e.g., "api call", "conv list", "msg post")

**Raw output (Slack API response only):**

Use the `--raw` flag to get the Slack API response without the envelope wrapper. This is useful for:
- Compatibility with existing scripts that expect raw Slack responses
- Piping output directly to `jq` or other tools
- Simpler output when metadata is not needed

```bash
# With envelope (default)
slack-rs conv list
# Output includes both "response" and "meta"

# Raw Slack API response only
slack-rs conv list --raw
# Output is the Slack API response without envelope

# Works with all commands
slack-rs api call conversations.list --raw
slack-rs msg post C123 "Hello" --raw
slack-rs search "query" --raw
```

**Accessing data with jq:**

```bash
# With envelope (default) - access response data
slack-rs conv list | jq '.response.channels[].name'

# With envelope - access metadata
slack-rs conv list | jq '.meta.command'

# Raw output - direct access (for backward compatibility)
slack-rs conv list --raw | jq '.channels[].name'
```

**Migration guide for existing scripts:**

If you have existing scripts that parse the output, you can:
1. Add `--raw` flag to maintain current behavior
2. Update scripts to extract from `.response` field
3. Optionally use `.meta` for additional context

```bash
# Old script (worked before, but now needs update)
CHANNEL=$(slack-rs conv list | jq -r '.channels[0].id')

# Option 1: Use --raw flag (quick fix)
CHANNEL=$(slack-rs conv list --raw | jq -r '.channels[0].id')

# Option 2: Extract from .response (recommended)
CHANNEL=$(slack-rs conv list | jq -r '.response.channels[0].id')

# Option 3: Use metadata too
RESULT=$(slack-rs conv list | jq -r '{channel: .response.channels[0].id, team: .meta.team_id}')
```

## Configuration

### Environment Variables

Only the following environment variables are supported by the current implementation. OAuth client credentials are configured via `slack-rs config oauth set` (not environment variables).

| Variable | Description | Default | Use Case |
|----------|-------------|---------|----------|
| `SLACKCLI_ALLOW_WRITE` | Control write operations (post/update/delete messages). Values: `true`, `1`, `yes` (allow) or `false`, `0`, `no` (deny) | `true` | Safety in production environments |
| `SLACKRS_KEYRING_PASSWORD` | Passphrase for encrypting/decrypting export files. Use strong passphrase (16+ chars). Alternative to `--passphrase-prompt` flag. | - | Automated backup/restore scripts |
| `SLACK_OAUTH_BASE_URL` | Custom OAuth base URL for testing or private Slack installations. Example: `https://custom-slack.example.com` | `https://slack.com` | Testing, enterprise Slack instances |

**Setting environment variables:**

```bash
# Example: Prevent accidental write operations
export SLACKCLI_ALLOW_WRITE=false

# Example: Non-interactive export/import passphrase
export SLACKRS_KEYRING_PASSWORD="your-secure-passphrase"

# Example: Use custom OAuth base URL (testing)
export SLACK_OAUTH_BASE_URL="https://slack.com"
```

### Profile Storage

- **Profile metadata**: `~/.config/slack-rs/profiles.json` (Linux/macOS) or `%APPDATA%\slack-rs\profiles.json` (Windows)
- **Sensitive credentials**: OS keyring (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows)

Each profile stores:
- **In JSON file**: `team_id`, `user_id`, `team_name`, `user_name`, `client_id`, `redirect_uri`, `scopes`
- **In OS keyring**: Access token and client secret (when saved via `config oauth set` or export/import)

### Write Operation Protection

Write operations (posting, updating, deleting messages, and managing reactions) are controlled by the `SLACKCLI_ALLOW_WRITE` environment variable:

- **Default behavior** (variable not set): Write operations are **allowed**
- **Deny write operations**: Set `SLACKCLI_ALLOW_WRITE=false` or `SLACKCLI_ALLOW_WRITE=0`
- **Explicitly allow**: Set `SLACKCLI_ALLOW_WRITE=true` or `SLACKCLI_ALLOW_WRITE=1`

**Example: Preventing accidental write operations**

```bash
# Deny all write operations
export SLACKCLI_ALLOW_WRITE=false

# This will fail with an error
slack-rs msg post C123456 "Hello"
# Error: Write operation denied. Set SLACKCLI_ALLOW_WRITE=true to enable write operations

# Re-enable write operations
export SLACKCLI_ALLOW_WRITE=true
slack-rs msg post C123456 "Hello"  # Now succeeds
```

## Security

### Credential Storage

**Access Tokens:**
All access tokens are stored securely in your operating system's credential manager:
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

Tokens are never stored in plain text files or logged to the console.

**Client Keys:**
- **Client IDs**: Stored in profile metadata file (`~/.config/slack-rs/profiles.json`). These are not considered sensitive as they're part of OAuth public flow.
- **Client Secrets**: Stored securely in OS keyring when provided (via `config oauth set` or during `auth login`). If not present in keyring, the CLI prompts for it.

### Profile Export/Import

Profile export/import enables secure backup and migration of your authentication profiles between machines or for disaster recovery.

#### What Gets Exported

When you export a profile, the following data is included in the encrypted file:
- **Profile metadata**: team ID, user ID, team name, user name, client ID
- **Access token**: OAuth access token for API calls
- **Client secret**: (Optional) OAuth client secret if you want to save it for convenience

#### Export Profiles

**Export a single profile:**

```bash
# With passphrase prompt (recommended)
slack-rs auth export --profile my-workspace --out backup.enc --passphrase-prompt

# With environment variable
export SLACKRS_KEYRING_PASSWORD="your-secure-passphrase"
slack-rs auth export --profile my-workspace --out backup.enc --yes
```

**Export all profiles:**

```bash
# Export all profiles at once
slack-rs auth export --all --out all-profiles.enc --passphrase-prompt

# Without confirmation prompt
export SLACKRS_KEYRING_PASSWORD="your-secure-passphrase"
slack-rs auth export --all --out all-profiles.enc --yes
```

#### Import Profiles

**Import a single profile:**

```bash
# Import with new profile name
slack-rs auth import --profile my-workspace --in backup.enc --passphrase-prompt

# Import all profiles from file (will prompt for each)
slack-rs auth import --all --in all-profiles.enc --passphrase-prompt
```

**Using environment variable for automation:**

```bash
export SLACKRS_KEYRING_PASSWORD="your-secure-passphrase"
slack-rs auth import --profile my-workspace --in backup.enc
slack-rs auth import --all --in all-profiles.enc
```

#### Security Details

- **Encryption**: AES-256-GCM (industry-standard authenticated encryption)
- **Key Derivation**: Argon2id with random salt (memory-hard, resistant to GPU attacks)
- **File Permissions**: Automatically set to `0600` (owner read/write only)
- **Passphrase**: Must be provided via `SLACKRS_KEYRING_PASSWORD` environment variable or `--passphrase-prompt`

#### Use Cases

1. **Backup**: Create encrypted backups of your profiles before system changes
2. **Migration**: Transfer profiles to a new machine or OS
3. **Team Sharing**: Share access credentials with team members (ensure secure passphrase exchange)
4. **Disaster Recovery**: Restore profiles after system failure or reinstallation

#### Best Practices

✅ **Do:**
- Use strong, unique passphrases (16+ characters with mixed case, numbers, symbols)
- Store exported files in secure locations (encrypted drives, password managers)
- Use `--passphrase-prompt` in scripts to avoid password in shell history
- Delete old export files after successful import

❌ **Don't:**
- Commit export files to version control (add `*.enc` to `.gitignore`)
- Share export files over unencrypted channels (use secure file transfer)
- Reuse passphrases across different export files
- Store passphrases in plain text files

**⚠️ Warning**: Exported files contain sensitive credentials including access tokens and potentially client secrets. Treat them like passwords and store securely.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding guidelines, and submission process.

## Roadmap

- [ ] Enhanced wrapper commands for common operations
- [ ] Support for slash commands
- [ ] Interactive mode for profile management
- [ ] Improved error messages with suggestions
- [ ] Internationalization (i18n) for English and Japanese

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [reqwest](https://github.com/seanmonstar/reqwest) for HTTP
- Secure storage with [keyring](https://github.com/hwchen/keyring-rs)
- OAuth implementation inspired by [oauth2-rs](https://github.com/ramosbugs/oauth2-rs)

## Support

- 🐛 [Report Issues](https://github.com/tumf/slack-rs/issues)
- 💬 [Discussions](https://github.com/tumf/slack-rs/discussions)
- 📖 [Slack API Documentation](https://api.slack.com/web)

---

**Note**: This is an unofficial tool and is not affiliated with or endorsed by Slack Technologies, Inc.
