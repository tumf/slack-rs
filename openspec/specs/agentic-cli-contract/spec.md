# agentic-cli-contract Specification

## Purpose
TBD - created by archiving change define-agentic-cli-contract. Update Purpose after archive.
## Requirements
### Requirement: `--non-interactive` はプロンプトを禁止し即時エラーにする
非対話モードでは stdin を読まず、確認・入力が必要な操作は即時に失敗することを SHALL とする。
#### Scenario: `msg delete` を `--non-interactive` で実行した場合
- `--yes` が無いときは確認プロンプトを出さずに失敗する
- stderr に「`--yes` が必要」であることと再実行例を示す
- 終了コードは 2（使用方法/確認不足）とする

### Requirement: TTY がない実行環境は非対話扱いになる
標準入力が TTY でない場合は、`--non-interactive` を暗黙に有効化することを SHALL とする。
#### Scenario: 標準入力が TTY でない環境で `react remove` を実行した場合
- `--yes` が無いときはプロンプトを出さずに失敗する
- 失敗の理由に「非対話環境である」旨が含まれる

### Requirement: `auth login` は非対話時に不足入力を列挙して失敗する
非対話モードでは OAuth 設定の不足入力を列挙し、入力待ちせずに終了することを SHALL とする。
#### Scenario: `auth login --non-interactive` で必要情報が不足している場合
- 不足している項目（例: `client_id`, `client_secret`, `redirect_uri`, `bot_scopes`/`user_scopes`）を列挙する
- 「どのフラグで指定できるか」を明示する
- stdin を読まずに終了する

