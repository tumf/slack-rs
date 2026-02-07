## MODIFIED Requirements

### Requirement: トークンはファイルに保存される

トークンは `~/.local/share/slack-rs/tokens.json` にファイルベースで保存されなければならない (MUST)。

ファイル形式は JSON で、キーと値のペアとして保存されなければならない (MUST)。

#### Scenario: トークンを保存してファイルに書き込まれる
- **WHEN** トークンを `set(key, token)` で保存する
- **THEN** `~/.local/share/slack-rs/tokens.json` にトークンが JSON 形式で書き込まれる

### Requirement: トークンファイルのパスは環境変数でオーバーライド可能

デフォルトのトークンファイルパスは `~/.local/share/slack-rs/tokens.json` でなければならない (MUST)。

環境変数 `SLACK_RS_TOKENS_PATH` が設定されている場合、そのパスを使用しなければならない (MUST)。

#### Scenario: デフォルトパスが使用される
- **WHEN** 環境変数 `SLACK_RS_TOKENS_PATH` が設定されていない
- **THEN** `~/.local/share/slack-rs/tokens.json` がトークンファイルパスとして使用される

#### Scenario: 環境変数でパスをオーバーライドできる
- **WHEN** 環境変数 `SLACK_RS_TOKENS_PATH=/tmp/test-tokens.json` が設定されている
- **THEN** `/tmp/test-tokens.json` がトークンファイルパスとして使用される

### Requirement: 旧トークンパスから新パスへ自動移行される

環境変数 `SLACK_RS_TOKENS_PATH` が未設定で、新パス `~/.local/share/slack-rs/tokens.json` が存在せず、旧パス `~/.config/slack-rs/tokens.json` が存在する場合、初期化時に旧ファイル内容を新パスへ移行しなければならない (MUST)。

移行後の読み書きは新パスに対して行われなければならない (MUST)。

#### Scenario: 旧パスのみ存在する場合に自動移行される
- **WHEN** 旧パスにのみ `tokens.json` が存在し、新パスには存在しない
- **THEN** `FileTokenStore` 初期化時に新パスへ同内容が作成される
- **AND** 以降の `get/set/delete` は新パスに対して動作する
