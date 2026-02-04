# auth-export-import Specification

## Purpose
Enables secure backup and restoration of authentication profiles through encrypted export/import, allowing users to transfer credentials across machines or create secure backups.
## Requirements
### Requirement: Provide export/import via CLI
Export/import of profiles MUST be executable from the CLI.
#### Scenario: Export command encrypts and saves profile
- **WHEN** `slackcli auth export --profile <name> --out <path>` is executed
- **THEN** the specified profile is encrypted and saved to the output path

### Requirement: Export requires encryption and does not output plaintext
Only encrypted binary MUST be generated, and plaintext authentication information MUST NOT be output. (MUST NOT)
#### Scenario: Export produces encrypted binary only
- **WHEN** `slackcli auth export` is executed
- **THEN** only encrypted binary is generated
- **AND** plain text JSON is not written to files or stdout

### Requirement: Passphrase is obtained from env or prompt
Passphrase MUST be obtainable from environment variables or interactive input. (MUST)
#### Scenario: Passphrase from env or prompt
- **WHEN** `--passphrase-env` is set and the environment variable exists
- **THEN** the passphrase is taken from the environment variable
- **WHEN** the environment variable does not exist
- **THEN** the passphrase is prompted interactively via `--passphrase-prompt`

### Requirement: Export requires confirmation for dangerous operation
Export MUST NOT execute without explicit consent. (MUST NOT)
#### Scenario: Export requires explicit confirmation
- **WHEN** `--yes` flag is not provided
- **THEN** a warning is displayed
- **AND** the export operation is aborted

### Requirement: Export file permissions are enforced to 0600
Saving MUST only be possible with secure file permissions. (MUST)
#### Scenario: File permissions enforced to 0600
- **WHEN** a new file is created
- **THEN** it is created with 0600 permissions
- **WHEN** an existing file has permissions other than 0600
- **THEN** an error is returned

### Requirement: Import writes back to keyring

インポート時に復元対象の機密情報はKeyringへ保存されることがMUST。

#### Scenario: Import decrypts and stores to keyring
- `slackcli auth import --in <path>` 実行時、OAuthクライアントシークレットが含まれていればKeyringへ保存される

### Requirement: Import applies safeguard on team_id conflict
A safeguard MUST be applied when the same team_id exists. (MUST)
#### Scenario: Conflict handling for duplicate team_id
- **WHEN** a profile with the same team_id already exists
- **THEN** the import fails by default
- **WHEN** both `--yes` and `--force` flags are provided
- **THEN** the existing profile is overwritten

### Requirement: Export/import format is resilient to future extensions
Compatible reading and writing MUST be possible. (MUST)
#### Scenario: Format versioning for future compatibility
- **WHEN** the payload is written
- **THEN** it includes a `format_version` field
- **WHEN** the payload is read
- **THEN** unknown fields are ignored

### Requirement: Do not output tokens to logs or stdout
Sensitive information MUST NOT appear in output paths. (MUST NOT)
#### Scenario: Sensitive tokens not exposed in output
- **WHEN** errors or debug output are generated
- **THEN** access_token and refresh_token are not included

### Requirement: Warnings and prompts can be switched via i18n
Warnings and prompts MUST be displayed in the specified language. (MUST)
#### Scenario: Language switching for warnings and prompts
- **WHEN** `--lang ja` or `--lang en` is specified
- **THEN** warnings and prompts are displayed in the specified language

### Requirement: Export/importにOAuthクレデンシャルを含める

エクスポート対象のプロファイルにOAuthクレデンシャルが存在する場合、暗号化ペイロードに含めることがMUST。

#### Scenario: Export includes OAuth credentials when available
- `client_id` が存在する場合、エクスポートペイロードに含まれる
- `client_secret` がKeyringに存在する場合、エクスポートペイロードに含まれる

### Requirement: OAuthクレデンシャルは設定ファイルに平文で保存しない

OAuthクライアントシークレットは設定ファイルに保存されないことがMUST。

#### Scenario: Import stores client_secret only in keyring
- import時に `client_secret` はKeyringへ保存され、設定ファイルには書き込まれない

