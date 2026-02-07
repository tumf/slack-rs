# doctor-diagnostics Delta Specification

## Purpose
診断情報を一括収集する `doctor` コマンドを提供し、認証・環境問題の切り分けを迅速化する。

## Requirements

### Requirement: doctor コマンドで診断情報を取得できる
`doctor` コマンドは、profile、token store backend/path、bot/user token の有無を表示しなければならない。(MUST)
`--profile` が指定された場合、そのプロファイル名で診断を実行しなければならない。(MUST)
`--profile` が未指定の場合、`default` プロファイルを使用しなければならない。(MUST)

#### Scenario: デフォルトプロファイルで doctor を実行する
- Given 有効な `default` プロファイルが存在する
- When `doctor` を実行する
- Then config path、token store backend、token store path、bot/user token の有無が表示される
- And token の値は表示されない

#### Scenario: 指定したプロファイルで doctor を実行する
- Given 有効な `work` プロファイルが存在する
- When `doctor --profile work` を実行する
- Then `work` プロファイルの診断情報が表示される

### Requirement: doctor --json で機械可読出力を提供する
`doctor --json` は、診断情報を JSON 形式で出力しなければならない。(MUST)
出力には `config_path`、`token_store` (backend, path)、`tokens` (bot_token_exists, user_token_exists) を含めなければならない。(MUST)

#### Scenario: JSON 出力で診断情報を取得する
- Given 有効なプロファイルが存在する
- When `doctor --json` を実行する
- Then JSON 形式で `config_path`、`token_store`、`tokens` が出力される
- And JSON パース可能である

### Requirement: doctor は token 値を出力しない
`doctor` コマンドの出力に、token の値（`xoxb-`, `xoxp-` で始まる文字列）を含めてはならない。(MUST NOT)
token の有無のみを boolean フラグ（`bot_token_exists`, `user_token_exists`）で示さなければならない。(MUST)

#### Scenario: 出力に token 値が含まれない
- Given token store に有効な bot/user token が保存されている
- When `doctor` または `doctor --json` を実行する
- Then 出力に `xoxb-` または `xoxp-` を含まない
- And `bot_token_exists` と `user_token_exists` のフラグのみが出力される

### Requirement: doctor にヘルプを追加する
`--help` 出力に `doctor` コマンドの説明を含めなければならない。(MUST)

#### Scenario: ヘルプに doctor が表示される
- When `--help` を表示する
- Then `doctor` コマンドの説明が含まれる
- And `--profile` と `--json` オプションの説明が含まれる
