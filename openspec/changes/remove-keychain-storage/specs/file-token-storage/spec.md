# file-token-storage Specification Delta

## MODIFIED Requirements

### Requirement: FileTokenStore is always enabled

FileTokenStore は常に有効であり、トークンストレージは必ず FileTokenStore を使用しなければならない (MUST)。

`SLACKRS_TOKEN_STORE` は使用してはならない (MUST NOT)。

#### Scenario: FileTokenStore が常時使用される
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- When トークンを保存する
- Then `~/.config/slack-rs/tokens.json` に書き込まれる
