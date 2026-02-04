# profile-oauth-credentials Specification

## Purpose
TBD - created by archiving change add-per-profile-oauth-credentials. Update Purpose after archive.
## Requirements
### Requirement: ログイン時のOAuthクレデンシャル取得は対話入力を優先する

ログイン時のクライアント情報は対話入力を前提とし、ユーザー操作で安全に入力できることがMUST。

#### Scenario: `--client-id` が未指定の場合に入力を促し、`client_secret` は常にプロンプトされる
- `slack-rs login` 実行時、`--client-id` が無い場合は標準入力でIDを求める
- 既存プロファイルに `client_id` が保存されている場合は入力を省略する
- `client_secret` は常に非表示入力で取得する
- `client_secret` をコマンドライン引数として受け付けない

### Requirement: プロファイルに `client_id` を保存し、`client_secret` はKeyringに保存する

プロファイルごとにOAuthクライアントIDを保持し、シークレットは設定ファイルに残さないことがMUST。

#### Scenario: ログイン成功後に設定ファイルとKeyringに保存される
- `profiles.json` に `client_id` が保存される
- `client_secret` はKeyringに保存され、設定ファイルには書き込まれない

### Requirement: 既存プロファイルは `client_id` 未設定でも読み込める

過去に保存された設定ファイルがそのまま読み込めることがMUST。

#### Scenario: 旧形式の `profiles.json` を読み込んでもエラーにならない
- `client_id` が欠落していても読み込みが成功する

