# file-token-storage Specification

## Purpose
Defines file-based token storage implementation. Tokens are stored in `~/.config/slack-rs/tokens.json` with file permissions (0600) for security.
## Requirements
### Requirement: Tokens are stored in files

Tokens MUST be stored in file-based storage at `~/.local/share/slack-rs/tokens.json`. (MUST)

The file format MUST be JSON, stored as key-value pairs. (MUST)

#### Scenario: Tokens are saved and written to file
- **WHEN** a token is saved with `set(key, token)`
- **THEN** the token is written to `~/.local/share/slack-rs/tokens.json` in JSON format

### Requirement: File permissions are set to 0600

On Unix-like systems, token file permissions MUST be set to 0600 (owner read/write only). (MUST)

On non-Unix systems such as Windows, permission setting MUST be skipped. (MUST)

#### Scenario: File permissions are set to 0600 on Unix systems
- **WHEN** a token is saved on a Unix system
- **THEN** `tokens.json` file permissions are set to 0600

#### Scenario: Permission setting failure returns an error
- **WHEN** file permission setting fails
- **THEN** a `StoreFailed` error is returned

### Requirement: Token file path can be overridden with environment variable

The default token file path MUST be `~/.local/share/slack-rs/tokens.json`. (MUST)

If the environment variable `SLACK_RS_TOKENS_PATH` is set, that path MUST be used. (MUST)

#### Scenario: Default path is used
- **WHEN** environment variable `SLACK_RS_TOKENS_PATH` is not set
- **THEN** `~/.local/share/slack-rs/tokens.json` is used as the token file path

#### Scenario: Path can be overridden with environment variable
- **WHEN** environment variable `SLACK_RS_TOKENS_PATH=/tmp/test-tokens.json` is set
- **THEN** `/tmp/test-tokens.json` is used as the token file path

### Requirement: Parent directory is created automatically if it does not exist

If the parent directory of the token file does not exist, it MUST be created automatically. (MUST)

If directory creation fails, an `IoError` MUST be returned. (MUST)

#### Scenario: Parent directory is created automatically
- **WHEN** a token is saved when `~/.config/slack-rs/` does not exist
- **THEN** the `~/.config/slack-rs/` directory is created automatically

#### Scenario: Directory creation failure returns an error
- **WHEN** parent directory creation fails
- **THEN** an `IoError` is returned

### Requirement: Tokens can be deleted

Tokens MUST be deletable with the `delete(key)` method. (MUST)

After deletion, calling `get(key)` with the same key MUST return a `NotFound` error. (MUST)

The file MUST also be updated when deleting. (MUST)

#### Scenario: Tokens can be deleted
- **WHEN** `delete("T123:U456")` is called after saving a token
- **THEN** the token is deleted and `get("T123:U456")` returns a `NotFound` error

#### Scenario: File is updated after deletion
- **WHEN** a token is deleted
- **THEN** the corresponding key is removed from `tokens.json`

### Requirement: Token existence can be checked

Token existence MUST be checkable with the `exists(key)` method. (MUST)

It MUST return `true` if the token exists and `false` if it does not. (MUST)

#### Scenario: Existing tokens return true
- **WHEN** `exists("T123:U456")` is called after saving a token
- **THEN** `true` is returned

#### Scenario: Non-existing tokens return false
- **WHEN** `exists("nonexistent")` is called with a key that has not been saved
- **THEN** `false` is returned

### Requirement: Existing token files can be loaded

When initializing `FileTokenStore`, if an existing `tokens.json` file exists, its contents MUST be loaded. (MUST)

If the file does not exist, it MUST be initialized with an empty token map. (MUST)

If JSON parsing of the file fails, an `IoError` MUST be returned. (MUST)

#### Scenario: Existing token file can be loaded
- **WHEN** `FileTokenStore` is initialized when existing tokens are saved in `tokens.json`
- **THEN** existing tokens are loaded and can be retrieved with `get()`

#### Scenario: File is initialized empty if it does not exist
- **WHEN** `FileTokenStore` is initialized when `tokens.json` does not exist
- **THEN** it is initialized with an empty token map

#### Scenario: JSON parse failure returns an error
- **WHEN** `tokens.json` is in invalid JSON format
- **THEN** an `IoError` is returned

### Requirement: Multiple tokens can be stored

Multiple tokens with different keys MUST be storable. (MUST)

Each token MUST be retrievable and deletable independently. (MUST)

#### Scenario: Multiple tokens can be stored and retrieved individually
- **WHEN** `set("T1:U1", "token1")` and `set("T2:U2", "token2")` are called
- **THEN** `get("T1:U1")` returns `"token1"` and `get("T2:U2")` returns `"token2"`

#### Scenario: Other tokens remain when one token is deleted
- **WHEN** one token is deleted after saving multiple tokens
- **THEN** the deleted token cannot be retrieved but other tokens can be retrieved

### Requirement: Automatic migration from legacy token path to new path

When the environment variable `SLACK_RS_TOKENS_PATH` is not set, the new path `~/.local/share/slack-rs/tokens.json` does not exist, and the legacy path `~/.config/slack-rs/tokens.json` exists, the legacy file contents MUST be migrated to the new path during initialization. (MUST)

After migration, read and write operations MUST be performed on the new path. (MUST)

#### Scenario: Automatic migration when only legacy path exists
- **WHEN** `tokens.json` exists only in the legacy path and not in the new path
- **THEN** the same content is created in the new path during `FileTokenStore` initialization
- **AND** subsequent `get/set/delete` operations work on the new path

## Requirements
### Requirement: FileTokenStore is always enabled

FileTokenStore is always enabled, and token storage MUST always use FileTokenStore. (MUST)

`SLACKRS_TOKEN_STORE` MUST NOT be used. (MUST NOT)

#### Scenario: FileTokenStore is always used
- Given `SLACKRS_TOKEN_STORE` is not set
- When saving a token
- Then it is written to `~/.config/slack-rs/tokens.json`

