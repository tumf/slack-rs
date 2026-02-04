# slack-rs

A Slack CLI tool written in Rust that provides comprehensive access to the Slack Web API using OAuth authentication.

[![CI](https://github.com/tumf/slack-rs/workflows/CI/badge.svg)](https://github.com/tumf/slack-rs/actions)
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

## Installation

### Prerequisites

- Rust 1.70+ (tested with 1.92.0)
- A Slack app with OAuth credentials ([create one here](https://api.slack.com/apps))

### Build from Source

```bash
git clone https://github.com/tumf/slack-rs.git
cd slack-rs
cargo build --release
```

The binary will be available at `target/release/slack-rs`.

### Install via Cargo

```bash
cargo install --path .
```

## Quick Start

### 1. Set Up OAuth Credentials

#### Creating a Slack App

1. **Navigate to Slack API**: Go to https://api.slack.com/apps
2. **Create an app**:
   - Click "Create New App"
   - Choose "From scratch"
   - Name your app (e.g., "My Slack CLI")
   - Select a development workspace
3. **Configure OAuth & Permissions**:
   - In the left sidebar, click "OAuth & Permissions"
   - Scroll to "Redirect URLs" section
   - Add redirect URL: `http://127.0.0.1:3000/callback`
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
   - Scroll to top of "OAuth & Permissions" page
   - Copy your **Client ID** (looks like `123456789012.1234567890123`)
   - Click "Show" and copy your **Client Secret** (looks like `abcdef1234567890abcdef1234567890`)

#### Providing Credentials

**Option A: Environment variables (recommended for single workspace):**

```bash
export SLACKRS_CLIENT_ID="123456789012.1234567890123"
export SLACKRS_CLIENT_SECRET="abcdef1234567890abcdef1234567890"
export SLACKRS_REDIRECT_URI="http://127.0.0.1:3000/callback"  # must match Slack app config
export SLACKRS_SCOPES="chat:write,users:read,channels:read"    # comma-separated, no spaces
```

**Option B: Command-line argument (recommended for multiple workspaces):**

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

**💡 Pro Tip**: Use Option B or C for per-profile credentials when managing multiple workspaces with different Slack apps.

### 2. Authenticate

**Login to your Slack workspace:**

```bash
# Method 1: Using environment variables
export SLACKRS_CLIENT_ID="123456789012.1234567890123"
export SLACKRS_CLIENT_SECRET="abcdef1234567890abcdef1234567890"
slack-rs auth login my-workspace

# Method 2: Provide client ID, prompt for secret
slack-rs auth login my-workspace --client-id 123456789012.1234567890123

# Method 3: Interactive - prompt for both
slack-rs auth login my-workspace
```

**What happens during login:**

1. **Credentials collected**: Client ID and secret are obtained (from args, env vars, or prompts)
2. **Browser opens**: OAuth authorization page opens automatically
3. **User authorization**: Click "Allow" to grant permissions to your app
4. **Callback handled**: Local server receives OAuth callback with authorization code
5. **Token exchange**: Code is exchanged for access token
6. **Secure storage**: Profile and token are saved securely
   - Profile metadata → `~/.config/slack-rs/profiles.json`
   - Access token → OS Keyring (Keychain/Secret Service/Credential Manager)

**After successful authentication:**

```
✓ Authentication successful!
Profile 'my-workspace' saved.
```

**Per-Profile Client Keys:**
- ✅ Each profile stores its own OAuth client ID
- 💾 Client ID saved in `~/.config/slack-rs/profiles.json`
- 🔒 Client secret prompted each time (not saved for security)
- 🔄 Subsequent logins reuse saved client ID automatically

### 3. Make API Calls

**Generic API call:**

```bash
slack-rs api call chat.postMessage channel=C123456 text="Hello from slack-rs!"
```

**Check authentication status:**

```bash
slack-rs auth status my-workspace
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
# Basic login (uses env vars or prompts)
slack-rs auth login [profile-name]

# Login with specific client ID
slack-rs auth login [profile-name] --client-id <client-id>

# Examples:
slack-rs auth login                           # Profile named "default"
slack-rs auth login my-team                   # Profile named "my-team"
slack-rs auth login dev --client-id 12345.67  # With explicit client ID
```

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

#### Per-Profile Client Keys

Each profile can store its own OAuth client ID, enabling flexible multi-workspace and multi-app workflows.

**Benefits:**
- ✅ **Different Slack apps per workspace**: Use separate apps for different teams
- ✅ **Development/Production separation**: Test with dev app, deploy with prod app
- ✅ **Granular permission control**: Different scopes for different profiles
- ✅ **Client rotation**: Update client IDs without affecting other profiles
- ✅ **Team collaboration**: Each team member can use their own Slack app

**How it works:**

| Step | Action | Storage Location |
|------|--------|------------------|
| 1️⃣ | Provide client ID via `--client-id`, env var, or prompt | In-memory |
| 2️⃣ | Authenticate via OAuth flow in browser | Slack |
| 3️⃣ | Client ID saved to profile metadata | `~/.config/slack-rs/profiles.json` |
| 4️⃣ | Access token saved securely | OS Keyring |
| 5️⃣ | On re-login, saved client ID is reused | Auto-loaded |

**Examples:**

```bash
# Scenario 1: Development workspace with dev app
slack-rs auth login dev-workspace --client-id 111111111111.222222222222
# ✓ Client ID 111111111111.222222222222 saved to profile "dev-workspace"

# Scenario 2: Production workspace with prod app
slack-rs auth login prod-workspace --client-id 333333333333.444444444444
# ✓ Client ID 333333333333.444444444444 saved to profile "prod-workspace"

# Scenario 3: Re-authenticate dev-workspace (reuses saved client ID)
slack-rs auth login dev-workspace
# ℹ Using saved client ID: 111111111111.222222222222
# Enter OAuth client secret: [prompts for secret only]

# Scenario 4: Update client ID for existing profile
slack-rs auth login dev-workspace --client-id 555555555555.666666666666
# ✓ Updated client ID for profile "dev-workspace"
```

**Security Notes:**
- **Client IDs**: Saved in profile JSON (not sensitive per OAuth 2.0 spec)
- **Client Secrets**: Never saved during normal login (prompted each time)
- **Exception**: Secrets are stored in OS keyring only during export/import for backup purposes

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

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SLACKRS_CLIENT_ID` | OAuth Client ID from your Slack app. Optional if using `--client-id` flag or interactive prompts. Format: `123456789012.123456789012` | - | No |
| `SLACKRS_CLIENT_SECRET` | OAuth Client Secret from your Slack app. Will be prompted securely if not set. | - | No |
| `SLACKRS_REDIRECT_URI` | OAuth callback URL. Must match the redirect URL configured in your Slack app settings. | `http://127.0.0.1:3000/callback` | No |
| `SLACKRS_SCOPES` | Comma-separated list of OAuth scopes (no spaces). Determines what API methods you can use. Examples: `chat:write,users:read,channels:read` | `chat:write,users:read` | No |
| `SLACKCLI_ALLOW_WRITE` | Control write operations (post/update/delete messages). Values: `true`, `1`, `yes` (allow) or `false`, `0`, `no` (deny) | `true` | No |
| `SLACKRS_KEYRING_PASSWORD` | Passphrase for encrypting/decrypting export files. Use strong passphrase (16+ chars). Alternative to `--passphrase-prompt` flag. | - | Only for export/import |
| `SLACK_OAUTH_BASE_URL` | Custom OAuth base URL for testing or private Slack installations. Example: `https://custom-slack.example.com` | `https://slack.com` | No |

**Setting environment variables:**

```bash
# Linux/macOS (current session)
export SLACKRS_CLIENT_ID="123456789012.123456789012"
export SLACKRS_CLIENT_SECRET="abcdef1234567890abcdef1234567890"

# Linux/macOS (permanent - add to ~/.bashrc or ~/.zshrc)
echo 'export SLACKRS_CLIENT_ID="123456789012.123456789012"' >> ~/.bashrc
echo 'export SLACKRS_CLIENT_SECRET="abcdef1234567890abcdef1234567890"' >> ~/.bashrc

# Windows (PowerShell)
$env:SLACKRS_CLIENT_ID = "123456789012.123456789012"
$env:SLACKRS_CLIENT_SECRET = "abcdef1234567890abcdef1234567890"

# Windows (permanent)
setx SLACKRS_CLIENT_ID "123456789012.123456789012"
setx SLACKRS_CLIENT_SECRET "abcdef1234567890abcdef1234567890"
```

### Profile Storage

- **Profile metadata** (includes client ID, team/user info): `~/.config/slack-rs/profiles.json` (Linux/macOS)
- **Sensitive credentials** (access tokens and client secrets): OS keyring (Keychain on macOS, Secret Service on Linux, Credential Manager on Windows)

Each profile stores:
- **In JSON file**: `team_id`, `user_id`, `team_name`, `user_name`, `client_id`
- **In OS keyring**: Access token and client secret (when saved via export/import)

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
- **Client Secrets**: Never saved to disk during normal operation. Always prompted when needed for authentication. Only stored in OS keyring when explicitly saved via export/import for backup purposes.

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

## Development

### Building

```bash
cargo build              # Debug build
cargo build --release    # Optimized build
```

### Testing

```bash
cargo test                                    # Run all tests
cargo test test_api_call_with_form_data      # Run specific test
cargo test --test api_integration_tests       # Run integration tests
cargo test -- --nocapture                     # Show println! output
```

### Linting and Formatting

```bash
cargo fmt                    # Format code
cargo fmt -- --check         # Check formatting
cargo clippy                 # Run linter
cargo clippy -- -D warnings  # Fail on warnings (CI standard)
```

### Code Coverage

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --verbose --all-features --workspace --timeout 120
```

## Project Structure

```
slack-rs/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library root
│   ├── api/              # Slack API client
│   ├── auth/             # Auth commands
│   ├── cli/              # CLI helpers
│   ├── commands/         # Wrapper commands
│   ├── oauth/            # OAuth flow (PKCE)
│   └── profile/          # Profile management
├── tests/                # Integration tests
├── Cargo.toml            # Dependencies
└── AGENTS.md             # Developer guidelines
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please ensure:
- Code passes `cargo fmt -- --check` and `cargo clippy -- -D warnings`
- All tests pass with `cargo test`
- New features include tests

See [AGENTS.md](AGENTS.md) for detailed coding guidelines.

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
