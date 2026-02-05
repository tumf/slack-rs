## ADDED Requirements

### Requirement: FileTokenStore is optional and enabled only via explicit opt-in

`FileTokenStore` はデフォルトの token store backend ではない (MUST NOT)。

`FileTokenStore` は環境変数 `SLACKRS_TOKEN_STORE=file` が設定された場合にのみ使用してよい (MAY)。

#### Scenario: `SLACKRS_TOKEN_STORE=file` のときのみ FileTokenStore が選択される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When token store backend を解決する
- Then `FileTokenStore` が選択される

### Requirement: In file mode, tokens are stored in tokens.json (unchanged)

file mode の場合、トークンは `~/.config/slack-rs/tokens.json` に保存されなければならない (MUST)。

#### Scenario: file mode でトークンを保存してファイルに書き込まれる
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When トークンを `set(key, token)` で保存する
- Then `~/.config/slack-rs/tokens.json` にトークンが JSON 形式で書き込まれる

### Requirement: tokens.json path override applies only in file mode

環境変数 `SLACK_RS_TOKENS_PATH` によるトークンファイルパスのオーバーライドは file mode の場合にのみ適用されなければならない (MUST)。

#### Scenario: file mode で `SLACK_RS_TOKENS_PATH` が優先される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- And `SLACK_RS_TOKENS_PATH=/tmp/test-tokens.json` が設定されている
- When トークンを保存する
- Then `/tmp/test-tokens.json` が使用される
