# oauth-login Specification

## MODIFIED Requirements
### Requirement: 必須設定が未設定の場合は開始しない
`SLACKRS_CLIENT_ID` または `SLACKRS_CLIENT_SECRET` が未設定の場合、OAuth フローを開始してはならない。(MUST NOT)
#### Scenario: 必須環境変数が不足している
- Given `SLACKRS_CLIENT_ID` が未設定である
- When `auth login` を実行する
- Then エラーで終了する
