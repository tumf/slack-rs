# file-token-storage Specification

## Purpose
ファイルベースのトークンストレージ実装を定義する。トークンを `~/.config/slack-rs/tokens.json` に保存し、ファイルパーミッション（0600）でセキュリティを確保する。

## ADDED Requirements

### Requirement: トークンはファイルに保存される

トークンは `~/.config/slack-rs/tokens.json` にファイルベースで保存されなければならない (MUST)。

ファイル形式は JSON で、キーと値のペアとして保存されなければならない (MUST)。

#### Scenario: トークンを保存してファイルに書き込まれる
- **WHEN** トークンを `set(key, token)` で保存する
- **THEN** `~/.config/slack-rs/tokens.json` にトークンが JSON 形式で書き込まれる

#### Scenario: 保存したトークンを取得できる
- **WHEN** トークンを `set("T123:U456", "xoxb-test-token")` で保存する
- **THEN** `get("T123:U456")` で同じトークン値が取得できる

### Requirement: ファイルパーミッションは 0600 に設定される

Unix 系システムでは、トークンファイルのパーミッションは 0600（所有者のみ読み書き可能）に設定されなければならない (MUST)。

Windows など Unix 以外のシステムでは、パーミッション設定はスキップされなければならない (MUST)。

#### Scenario: Unix システムでファイルパーミッションが 0600 に設定される
- **WHEN** Unix システムでトークンを保存する
- **THEN** `tokens.json` のファイルパーミッションが 0600 になる

#### Scenario: パーミッション設定失敗時はエラーを返す
- **WHEN** ファイルパーミッションの設定に失敗する
- **THEN** `StoreFailed` エラーが返される

### Requirement: トークンファイルのパスは環境変数でオーバーライド可能

デフォルトのトークンファイルパスは `~/.config/slack-rs/tokens.json` でなければならない (MUST)。

環境変数 `SLACK_RS_TOKENS_PATH` が設定されている場合、そのパスを使用しなければならない (MUST)。

#### Scenario: デフォルトパスが使用される
- **WHEN** 環境変数 `SLACK_RS_TOKENS_PATH` が設定されていない
- **THEN** `~/.config/slack-rs/tokens.json` がトークンファイルパスとして使用される

#### Scenario: 環境変数でパスをオーバーライドできる
- **WHEN** 環境変数 `SLACK_RS_TOKENS_PATH=/tmp/test-tokens.json` が設定されている
- **THEN** `/tmp/test-tokens.json` がトークンファイルパスとして使用される

### Requirement: 親ディレクトリが存在しない場合は自動作成される

トークンファイルの親ディレクトリが存在しない場合、自動的に作成されなければならない (MUST)。

ディレクトリ作成に失敗した場合は、`IoError` を返さなければならない (MUST)。

#### Scenario: 親ディレクトリが自動作成される
- **WHEN** `~/.config/slack-rs/` が存在しない状態でトークンを保存する
- **THEN** `~/.config/slack-rs/` ディレクトリが自動的に作成される

#### Scenario: ディレクトリ作成失敗時はエラーを返す
- **WHEN** 親ディレクトリの作成に失敗する
- **THEN** `IoError` が返される

### Requirement: トークンの削除が可能

トークンは `delete(key)` メソッドで削除できなければならない (MUST)。

削除後、同じキーで `get(key)` を呼び出すと `NotFound` エラーが返されなければならない (MUST)。

削除時にファイルも更新されなければならない (MUST)。

#### Scenario: トークンを削除できる
- **WHEN** トークンを保存した後に `delete("T123:U456")` を呼び出す
- **THEN** トークンが削除され、`get("T123:U456")` が `NotFound` エラーを返す

#### Scenario: 削除後にファイルが更新される
- **WHEN** トークンを削除する
- **THEN** `tokens.json` から該当のキーが削除される

### Requirement: トークンの存在確認が可能

トークンの存在は `exists(key)` メソッドで確認できなければならない (MUST)。

トークンが存在する場合は `true`、存在しない場合は `false` を返さなければならない (MUST)。

#### Scenario: 存在するトークンは true を返す
- **WHEN** トークンを保存した後に `exists("T123:U456")` を呼び出す
- **THEN** `true` が返される

#### Scenario: 存在しないトークンは false を返す
- **WHEN** 保存されていないキーで `exists("nonexistent")` を呼び出す
- **THEN** `false` が返される

### Requirement: 既存のトークンファイルを読み込める

`FileTokenStore` の初期化時、既存の `tokens.json` ファイルが存在する場合は、その内容を読み込まなければならない (MUST)。

ファイルが存在しない場合は、空のトークンマップで初期化されなければならない (MUST)。

ファイルの JSON パースに失敗した場合は、`IoError` を返さなければならない (MUST)。

#### Scenario: 既存のトークンファイルを読み込める
- **WHEN** `tokens.json` に既存のトークンが保存されている状態で `FileTokenStore` を初期化する
- **THEN** 既存のトークンが読み込まれ、`get()` で取得できる

#### Scenario: ファイルが存在しない場合は空で初期化される
- **WHEN** `tokens.json` が存在しない状態で `FileTokenStore` を初期化する
- **THEN** 空のトークンマップで初期化される

#### Scenario: JSON パース失敗時はエラーを返す
- **WHEN** `tokens.json` が不正な JSON 形式である
- **THEN** `IoError` が返される

### Requirement: 複数のトークンを保存できる

複数の異なるキーでトークンを保存できなければならない (MUST)。

各トークンは独立して取得・削除できなければならない (MUST)。

#### Scenario: 複数のトークンを保存して個別に取得できる
- **WHEN** `set("T1:U1", "token1")` と `set("T2:U2", "token2")` を呼び出す
- **THEN** `get("T1:U1")` は `"token1"` を返し、`get("T2:U2")` は `"token2"` を返す

#### Scenario: 一つのトークンを削除しても他のトークンは残る
- **WHEN** 複数のトークンを保存した後、一つを削除する
- **THEN** 削除したトークンは取得できないが、他のトークンは取得できる
## Requirements
### Requirement: FileTokenStore is always enabled

FileTokenStore は常に有効であり、トークンストレージは必ず FileTokenStore を使用しなければならない (MUST)。

`SLACKRS_TOKEN_STORE` は使用してはならない (MUST NOT)。

#### Scenario: FileTokenStore が常時使用される
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- When トークンを保存する
- Then `~/.config/slack-rs/tokens.json` に書き込まれる

