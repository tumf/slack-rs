## MODIFIED Requirements

### Requirement: `auth status` は token store backend を表示する

`auth status` は現在選択されている token store backend（`keyring` または `file`）を表示しなければならない。(MUST)
`file` backend の場合は保存先パスを表示しなければならない。(MUST)
表示される保存先パスは `FileTokenStore` の実際のパス解決結果と一致しなければならない。(MUST)

#### Scenario: `XDG_DATA_HOME` 設定時に解決済み保存先が表示される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- And `SLACK_RS_TOKENS_PATH` は未設定で `XDG_DATA_HOME=/tmp/data` が設定されている
- When `auth status` を実行する
- Then `Token Store: file` が表示される
- And `/tmp/data/slack-rs/tokens.json` が保存先として表示される
