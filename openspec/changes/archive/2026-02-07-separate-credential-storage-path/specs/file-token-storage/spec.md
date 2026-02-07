## MODIFIED Requirements

### Requirement: Tokens are stored in files

Tokens MUST be stored in file-based storage at `~/.local/share/slack-rs/tokens.json`. (MUST)

The file format MUST be JSON, stored as key-value pairs. (MUST)

#### Scenario: Tokens are saved and written to file
- **WHEN** a token is saved with `set(key, token)`
- **THEN** the token is written to `~/.local/share/slack-rs/tokens.json` in JSON format

### Requirement: Token file path can be overridden with environment variable

The default token file path MUST be `~/.local/share/slack-rs/tokens.json`. (MUST)

If the environment variable `SLACK_RS_TOKENS_PATH` is set, that path MUST be used. (MUST)

#### Scenario: Default path is used
- **WHEN** environment variable `SLACK_RS_TOKENS_PATH` is not set
- **THEN** `~/.local/share/slack-rs/tokens.json` is used as the token file path

#### Scenario: Path can be overridden with environment variable
- **WHEN** environment variable `SLACK_RS_TOKENS_PATH=/tmp/test-tokens.json` is set
- **THEN** `/tmp/test-tokens.json` is used as the token file path

## ADDED Requirements

### Requirement: Automatic migration from legacy token path to new path

When the environment variable `SLACK_RS_TOKENS_PATH` is not set, the new path `~/.local/share/slack-rs/tokens.json` does not exist, and the legacy path `~/.config/slack-rs/tokens.json` exists, the legacy file contents MUST be migrated to the new path during initialization. (MUST)

After migration, read and write operations MUST be performed on the new path. (MUST)

#### Scenario: Automatic migration when only legacy path exists
- **WHEN** `tokens.json` exists only in the legacy path and not in the new path
- **THEN** the same content is created in the new path during `FileTokenStore` initialization
- **AND** subsequent `get/set/delete` operations work on the new path
