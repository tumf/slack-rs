# auth-export-import

## ADDED Requirements

### Requirement: CLI で export/import を提供する
CLI から profile の export/import を実行できることを MUST とする。
#### Scenario: Export command encrypts and saves profile
- **WHEN** `slackcli auth export --profile <name> --out <path>` is executed
- **THEN** the specified profile is encrypted and saved to the output path

### Requirement: export は暗号化必須で平文を出力しない
暗号化されたバイナリのみを生成し、平文の認証情報は出力しないことを MUST とする。
#### Scenario: Export produces encrypted binary only
- **WHEN** `slackcli auth export` is executed
- **THEN** only encrypted binary is generated
- **AND** plain text JSON is not written to files or stdout

### Requirement: passphrase は env または prompt から取得する
パスフレーズは環境変数または対話入力から取得できることを MUST とする。
#### Scenario: Passphrase from env or prompt
- **WHEN** `--passphrase-env` is set and the environment variable exists
- **THEN** the passphrase is taken from the environment variable
- **WHEN** the environment variable does not exist
- **THEN** the passphrase is prompted interactively via `--passphrase-prompt`

### Requirement: export は危険操作の確認を必須にする
export は明示的な同意が無い場合に実行しないことを MUST とする。
#### Scenario: Export requires explicit confirmation
- **WHEN** `--yes` flag is not provided
- **THEN** a warning is displayed
- **AND** the export operation is aborted

### Requirement: export のファイル権限は 0600 を強制する
安全なファイル権限でのみ保存できることを MUST とする。
#### Scenario: File permissions enforced to 0600
- **WHEN** a new file is created
- **THEN** it is created with 0600 permissions
- **WHEN** an existing file has permissions other than 0600
- **THEN** an error is returned

### Requirement: import は keyring に書き戻す
復号した認証情報を OS keyring に保存できることを MUST とする。
#### Scenario: Import decrypts and stores to keyring
- **WHEN** `slackcli auth import --in <path>` is executed
- **THEN** the encrypted file is decrypted
- **AND** the profile is saved to the OS keyring

### Requirement: import は team_id 競合時に安全装置を適用する
同一 team_id が存在する場合は安全装置が働くことを MUST とする。
#### Scenario: Conflict handling for duplicate team_id
- **WHEN** a profile with the same team_id already exists
- **THEN** the import fails by default
- **WHEN** both `--yes` and `--force` flags are provided
- **THEN** the existing profile is overwritten

### Requirement: export/import のフォーマットは将来拡張に耐える
互換性のある読み書きができることを MUST とする。
#### Scenario: Format versioning for future compatibility
- **WHEN** the payload is written
- **THEN** it includes a `format_version` field
- **WHEN** the payload is read
- **THEN** unknown fields are ignored

### Requirement: token をログや標準出力に出さない
機密情報が出力経路に現れないことを MUST とする。
#### Scenario: Sensitive tokens not exposed in output
- **WHEN** errors or debug output are generated
- **THEN** access_token and refresh_token are not included

### Requirement: i18n により警告・プロンプトを切り替えられる
指定言語で警告とプロンプトが表示されることを MUST とする。
#### Scenario: Language switching for warnings and prompts
- **WHEN** `--lang ja` or `--lang en` is specified
- **THEN** warnings and prompts are displayed in the specified language
