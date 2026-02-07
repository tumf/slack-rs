# profiles-and-token-store Specification Delta

## MODIFIED Requirements

### Requirement: Tokens are saved in file-based storage and not in configuration file

トークン（bot/user）および OAuth `client_secret` は `profiles.json` ではなく、FileTokenStore に保存されなければならない (MUST)。

トークンストレージは常に FileTokenStore を使用しなければならない (MUST)。

`SLACKRS_TOKEN_STORE` は使用してはならない (MUST NOT)。

#### Scenario: 常に FileTokenStore が使用される
- Given 環境変数が設定されていない
- When トークンストレージを初期化する
- Then FileTokenStore が選択される
