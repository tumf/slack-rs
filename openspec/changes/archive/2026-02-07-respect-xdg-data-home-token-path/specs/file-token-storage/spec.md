## MODIFIED Requirements

### Requirement: Token file path can be overridden with environment variable

The default token file path MUST resolve in the following order. (MUST)

1. If `SLACK_RS_TOKENS_PATH` is set, use it.
2. Else if `XDG_DATA_HOME` is set to a non-empty value, use `$XDG_DATA_HOME/slack-rs/tokens.json`.
3. Else, use `~/.local/share/slack-rs/tokens.json`.

#### Scenario: `XDG_DATA_HOME` が設定されている場合はその配下を使う
- **WHEN** `SLACK_RS_TOKENS_PATH` が未設定で `XDG_DATA_HOME=/tmp/data` が設定されている
- **THEN** token file path は `/tmp/data/slack-rs/tokens.json` になる

#### Scenario: `SLACK_RS_TOKENS_PATH` は `XDG_DATA_HOME` より優先される
- **WHEN** `SLACK_RS_TOKENS_PATH=/tmp/override.json` と `XDG_DATA_HOME=/tmp/data` の両方が設定されている
- **THEN** token file path は `/tmp/override.json` になる

#### Scenario: `XDG_DATA_HOME` が未設定または空値の場合は従来フォールバックを使う
- **WHEN** `SLACK_RS_TOKENS_PATH` が未設定で `XDG_DATA_HOME` が未設定または空値である
- **THEN** token file path は `~/.local/share/slack-rs/tokens.json` になる
