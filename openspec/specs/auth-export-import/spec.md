# auth-export-import Specification

## Purpose
Enables secure backup and restoration of authentication profiles through encrypted export/import, allowing users to transfer credentials across machines or create secure backups.
## Requirements
### Requirement: Provide export/import via CLI
Export/import of profiles MUST be executable from the CLI. `auth export --all` 実行時は、トークン未保存のプロファイルを警告してスキップし、他のプロファイルのエクスポートを継続しなければならない。(MUST)

#### Scenario: Export command encrypts and saves profile
- **WHEN** `slack-rs auth export --profile <name> --out <path>` is executed
- **THEN** the specified profile is encrypted and saved to the output path

#### Scenario: Export all skips profiles without tokens
- **WHEN** `slack-rs auth export --all --out <path>` is executed
- **AND** some profiles have no stored token
- **THEN** profiles with tokens are encrypted and saved to the output path
- **AND** profiles without tokens are reported as warnings and skipped

#### Scenario: Export all fails when no exportable profiles
- **WHEN** `slack-rs auth export --all --out <path>` is executed
- **AND** no profiles with tokens are available
- **THEN** the export operation is aborted with a clear error

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
`auth import` は衝突時の処理結果を実行後に報告しなければならない。(MUST)

`--yes` や `--force` の有無にかかわらず、更新・スキップ・上書きの件数と対象を取得できなければならない。(MUST)

`--json` 指定時は profile 単位の結果を機械可読形式で返さなければならない。(MUST)

#### Scenario: `--force --json` で profile 単位の結果を取得できる
- Given 衝突する profile を含む import ファイルがある
- When `auth import --force --json` を実行する
- Then 出力 JSON には profile ごとの `action` が含まれる
- And `updated` / `skipped` / `overwritten` の集計が取得できる

#### Scenario: テキスト出力でサマリと詳細が表示される
- Given import 可能な profile が含まれるファイルがある
- When `auth import --in <file> --yes` を実行する
- Then `Import Summary:` セクションに `Total`, `Updated`, `Skipped`, `Overwritten` 件数が表示される
- And `Profile Details:` セクションに各 profile の名前、action、理由が表示される

#### Scenario: JSON 出力で機械可読な結果が返される
- Given import 可能な profile が含まれるファイルがある
- When `auth import --in <file> --yes --json` を実行する
- Then 出力は有効な JSON フォーマットである
- And `profiles` 配列に各 profile の `profile_name`, `action`, `reason` が含まれる
- And `summary` オブジェクトに `total`, `updated`, `skipped`, `overwritten` が含まれる
- And `action` は `"updated"`, `"skipped"`, `"overwritten"` のいずれかである

#### Scenario: profile が新規追加される場合 action は updated
- Given 存在しない profile 名の import データがある
- When `auth import --in <file> --yes` を実行する
- Then その profile の action は `updated` である
- And reason には "New profile imported" が含まれる

#### Scenario: 同じ team_id の profile を更新する場合 action は updated
- Given 既存 profile と同じ team_id の import データがある
- When `auth import --in <file> --yes` を実行する (--force なし)
- Then その profile の action は `updated` である
- And reason には "Updated existing profile" と team_id が含まれる

#### Scenario: team_id が衝突する profile は --force なしで skipped
- Given 異なる profile 名だが同じ team_id の import データがある
- When `auth import --in <file> --yes` を実行する (--force なし)
- Then その profile の action は `skipped` である
- And reason には team_id conflict の情報が含まれる

#### Scenario: --force 指定時は衝突する profile が overwritten
- Given 既存 profile と衝突する import データがある
- When `auth import --in <file> --yes --force` を実行する
- Then その profile の action は `overwritten` である
- And reason には "Overwritten" の情報が含まれる

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
`auth export` と `auth import` は共有引数パーサを使う場合でも既存フラグ互換を維持しなければならない。(MUST)

`-h` および `--help` は unknown option として失敗してはならず、サブコマンド固有ヘルプを表示して終了コード 0 で終了しなければならない。(MUST)

#### Scenario: `auth export` / `auth import` のヘルプフラグが成功する
- **WHEN** `slack-rs auth export -h` または `slack-rs auth export --help` を実行する
- **THEN** export サブコマンドの usage/options が表示される
- **AND** 終了コードは 0 になる
- **WHEN** `slack-rs auth import -h` または `slack-rs auth import --help` を実行する
- **THEN** import サブコマンドの usage/options が表示される
- **AND** 終了コードは 0 になる

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

