# auth-export-import

## ADDED Requirements

### Requirement: CLI で export/import を提供する
CLI から profile の export/import を実行できることを MUST とする。
#### Scenario: `slackcli auth export --profile <name> --out <path>` を実行すると、指定 profile を暗号化して保存する

### Requirement: export は暗号化必須で平文を出力しない
暗号化されたバイナリのみを生成し、平文の認証情報は出力しないことを MUST とする。
#### Scenario: `slackcli auth export` 実行時に暗号化されたバイナリのみを生成し、平文 JSON はファイルにも標準出力にも出さない

### Requirement: passphrase は env または prompt から取得する
パスフレーズは環境変数または対話入力から取得できることを MUST とする。
#### Scenario: `--passphrase-env` が設定され、環境変数に値があればそれを使用し、無ければ `--passphrase-prompt` により対話入力する

### Requirement: export は危険操作の確認を必須にする
export は明示的な同意が無い場合に実行しないことを MUST とする。
#### Scenario: `--yes` が無い場合は警告を表示して export を中止する

### Requirement: export のファイル権限は 0600 を強制する
安全なファイル権限でのみ保存できることを MUST とする。
#### Scenario: 新規作成時は 0600 で作成し、既存ファイルの権限が 0600 以外の場合はエラーにする

### Requirement: import は keyring に書き戻す
復号した認証情報を OS keyring に保存できることを MUST とする。
#### Scenario: `slackcli auth import --in <path>` で暗号化ファイルを復号し、keyring に profile を保存する

### Requirement: import は team_id 競合時に安全装置を適用する
同一 team_id が存在する場合は安全装置が働くことを MUST とする。
#### Scenario: 同一 team_id の profile が存在する場合はデフォルトで失敗し、`--yes` + `--force` で上書きできる

### Requirement: export/import のフォーマットは将来拡張に耐える
互換性のある読み書きができることを MUST とする。
#### Scenario: ペイロードに `format_version` を持ち、unknown field を無視して読み込みできる

### Requirement: token をログや標準出力に出さない
機密情報が出力経路に現れないことを MUST とする。
#### Scenario: エラーやデバッグ出力に access_token/refresh_token が含まれない

### Requirement: i18n により警告・プロンプトを切り替えられる
指定言語で警告とプロンプトが表示されることを MUST とする。
#### Scenario: `--lang ja`/`--lang en` で警告・プロンプトの言語が切り替わる
