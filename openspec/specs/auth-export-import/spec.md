# auth-export-import Specification

## Purpose
Enables secure backup and restoration of authentication profiles through encrypted export/import, allowing users to transfer credentials across machines or create secure backups.
## Requirements
### Requirement: Provide export/import via CLI
Export/import of profiles MUST be executable from the CLI.
#### Scenario: Export command encrypts and saves profile
- **WHEN** `slack-rs auth export --profile <name> --out <path>` is executed
- **THEN** the specified profile is encrypted and saved to the output path

### Requirement: Export requires encryption and does not output plaintext
Only encrypted binary MUST be generated, and plaintext authentication information MUST NOT be output. (MUST NOT)
#### Scenario: Export produces encrypted binary only
- **WHEN** `slack-rs auth export` is executed
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
- `slack-rs auth import --in <path>` 実行時、OAuthクライアントシークレットが含まれていればKeyringへ保存される

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

### Requirement: Unified argument parsing preserves existing behavior
`auth export` and `auth import` MUST maintain existing flag, confirmation, and error behavior even when using a shared argument parser. (MUST)

#### Scenario: Shared parser maintains compatibility
- **GIVEN** existing `auth export`/`auth import` argument sets are used
- **WHEN** each command is executed
- **THEN** the same confirmation flows and error conditions apply as before

### Requirement: `auth import --dry-run` は書き込みなしで適用計画を表示する
`auth import --dry-run` は import 判定を実行するが、設定ファイルおよび token store への書き込みを行ってはならない。(MUST NOT)

dry-run 実行時は profile 単位の予定 action を出力しなければならない。(MUST)

#### Scenario: dry-run では書き込みせず予定のみ表示する
- Given 既存 profile と衝突する import データがある
- When `auth import --dry-run` を実行する
- Then `profiles.json` と token store の内容は変更されない
- And 各 profile の予定 action が表示される

### Requirement: dry-run は予定 action (created/updated/skipped/overwritten) を明示する
dry-run 実行時の出力は、各 profile に対する予定 action を含まなければならない。(MUST)

action は以下のいずれかでなければならない。(MUST)
- `created`: 新規 profile として作成予定
- `updated`: 既存 profile (同一 team_id) を更新予定
- `skipped`: 衝突のためスキップ予定 (--force なし)
- `overwritten`: 衝突だが上書き予定 (--force あり)

#### Scenario: 各 profile の action が表示される
- Given 新規 profile、更新 profile、衝突 profile を含む import データ
- When `auth import --dry-run` を実行する
- Then 各 profile に対する action (created/updated/skipped/overwritten) が表示される
- And action の理由 (reason) も表示される

### Requirement: `--dry-run --json` は機械可読の予定結果を返す
`--json` フラグと `--dry-run` の併用時、人間可読形式ではなく JSON 形式で予定結果を出力しなければならない。(MUST)

JSON 出力は以下の構造を持たなければならない。(MUST)
- `dry_run`: boolean (true/false)
- `profiles`: array of objects
  - `profile_name`: string
  - `action`: "created" | "updated" | "skipped" | "overwritten"
  - `team_id`: string
  - `user_id`: string
  - `reason`: string | null

#### Scenario: JSON 形式で予定結果を返す
- Given import データ
- When `auth import --dry-run --json` を実行する
- Then JSON 形式で予定結果が出力される
- And 各 profile の action, team_id, user_id, reason が含まれる

### Requirement: `--force` との併用時は上書き予定を報告する
`--dry-run` と `--force` を併用した場合、衝突する profile は `overwritten` として報告されなければならない。(MUST)

#### Scenario: force 時は上書き予定として報告
- Given 既存 profile と team_id が異なる import データ
- When `auth import --dry-run --force` を実行する
- Then 衝突する profile の action は `overwritten` となる
- And 実際の書き込みは行われない

