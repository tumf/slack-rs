# slack-rs

Slack CLI tool (Rust) — OAuth authentication, multi-profile, full Slack Web API access.

[![CI](https://github.com/tumf/slack-rs/workflows/CI/badge.svg)](https://github.com/tumf/slack-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/slack-rs.svg)](https://crates.io/crates/slack-rs)
[![Documentation](https://docs.rs/slack-rs/badge.svg)](https://docs.rs/slack-rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Designed following [Agentic CLI Design](https://dev.to/tumf/agentic-cli-design-7-principles-for-designing-cli-as-a-protocol-for-ai-agents-2c10) principles — structured JSON output, non-interactive operation, safe-by-default.

Key features:
- OAuth PKCE authentication with Cloudflare Tunnel support
- Multiple workspace profiles with independent credentials
- Generic API access + convenience wrapper commands
- Smart retry with exponential backoff and rate limit handling
- Encrypted profile export/import (AES-256-GCM + Argon2id)

## Installation

```bash
cargo install slack-rs        # From crates.io (recommended)
brew install tumf/tap/slack_rs     # Homebrew (tap)
```

Or build from source: `git clone`, `cargo build --release`. See [docs/authentication.md](docs/authentication.md) for prerequisites (Rust 1.70+, Slack App credentials).

## Agent Skills

Install embedded agent skill documentation for OpenCode/agent runtimes:

```bash
slack-rs install-skills           # → ./.agents/skills/slack-rs/
slack-rs install-skills --global  # → ~/.agents/skills/slack-rs/
```

## Quick Start

```bash
# 1. Authenticate (Cloudflare Tunnel — simplest)
slack-rs auth login my-workspace --cloudflared

# 2. Call any Slack API method
slack-rs api call chat.postMessage channel=C123 text="Hello!"

# 3. Use wrapper commands
slack-rs msg post C123 "Hello!"
slack-rs conv list
slack-rs search "quarterly report" count=10
```

For detailed setup (manual OAuth, remote auth, credential export/import), see [docs/authentication.md](docs/authentication.md).

## Usage

### API Calls

```bash
# Generic — call any Slack Web API method
slack-rs api call <method> [key=value...]
slack-rs api call users.info user=U123456
slack-rs api call conversations.history channel=C123456 limit=50

# Form-encoded arguments
slack-rs api call chat.postMessage channel=C123 text="Hello" thread_ts=1234567.123
```

### Wrapper Commands

| Command | Description |
|---------|-------------|
| `msg post <channel> <text>` | Post a message |
| `msg update <channel> <ts> <text>` | Update a message |
| `msg delete <channel> <ts>` | Delete a message |
| `conv list` | List conversations |
| `conv history <channel>` | Get conversation history |
| `conv search <query>` | Search conversations |
| `conv select <query>` | Interactive search with zoxide |
| `search <query>` | Search messages and files |
| `users info <user>` | Get user info |
| `thread get <channel> <ts>` | Get thread replies |
| `react add <channel> <ts> :emoji:` | Add reaction |
| `react remove <channel> <ts> :emoji:` | Remove reaction |
| `file upload <channel> <path>` | Upload a file |
| `file download <url>` | Download a file |

### Auth Commands (Quick Reference)

```bash
slack-rs auth login [profile] --cloudflared   # Login with tunnel
slack-rs auth status [profile]                # Check auth status
slack-rs auth list                            # List all profiles
slack-rs auth rename <old> <new>              # Rename profile
slack-rs auth logout <profile>                # Remove profile
slack-rs config oauth set/show/delete <profile>  # Manage OAuth config
```

Full auth guide: [docs/authentication.md](docs/authentication.md)

### Output Format

All commands output JSON with a unified envelope. Use `--raw` for raw Slack API response only.

```json
{
  "response": { "ok": true, "channels": [...] },
  "meta": {
    "profile_name": "default",
    "team_id": "T123ABC",
    "user_id": "U456DEF",
    "method": "conversations.list",
    "command": "conv list"
  }
}
```

```bash
slack-rs conv list --raw | jq '.channels[].name'      # Raw mode
slack-rs conv list | jq '.response.channels[].name'   # Default
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SLACKCLI_ALLOW_WRITE` | Control write ops (`true`/`false`) | `true` |
| `SLACK_OAUTH_BASE_URL` | Custom OAuth base URL (enterprise) | `https://slack.com` |

### Profile Storage

- `~/.config/slack-rs/profiles.json` — profile metadata (team, user, scopes)
- `~/.config/slack-rs/tokens.json` — access tokens + secrets (0600 permissions)

Each profile stores independent OAuth config. See [docs/config-and-storage.md](docs/config-and-storage.md) for schema details.

## Security

- **Write protection**: Set `SLACKCLI_ALLOW_WRITE=false` to prevent accidental writes
- **Tokens**: Stored in file-based storage (`~/.config/slack-rs/tokens.json`, 0600), never logged
- **Export/Import**: AES-256-GCM encryption with Argon2id key derivation
- **Rate limiting**: Automatic retry with exponential backoff + jitter

For full security specification, see [docs/security.md](docs/security.md).

## Development

This project uses [prek](https://prek.j178.dev/) for git hooks:

```bash
prek install          # Enable hooks
prek run --all-files  # Run all hooks manually
```

Hooks: `cargo fmt`, `cargo clippy`, trailing whitespace, file endings, YAML/TOML validation.

## Contributing

Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Roadmap

- Enhanced wrapper commands
- Slash command support
- Interactive profile management
- Internationalization (English / Japanese)

## License

MIT — see [LICENSE](LICENSE).

## Acknowledgments

Built with [Rust](https://www.rust-lang.org/), [reqwest](https://github.com/seanmonstar/reqwest), OAuth inspired by [oauth2-rs](https://github.com/ramosbugs/oauth2-rs).

---

**Note**: Unofficial tool, not affiliated with Slack Technologies, Inc.
