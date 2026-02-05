## ADDED Requirements
### Requirement: `auth status` は token store backend を表示する
`auth status` は現在選択されている token store backend（`keyring` または `file`）を表示しなければならない。(MUST)
`file` backend の場合は保存先パスを表示しなければならない。(MUST)

#### Scenario: file backend の保存先が表示される
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- When `auth status` を実行する
- Then `Token Store: file` が表示される
- And tokens.json の保存先パスが表示される

### Requirement: 別 backend に token がある場合は案内を出す
`auth status` が keyring backend を参照していて bot/user token が見つからない場合、file backend に該当キーが存在するなら `SLACKRS_TOKEN_STORE=file` を設定する案内を表示しなければならない。(MUST)

#### Scenario: keyring に存在せず file に存在する場合の案内
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And keyring backend では bot/user token が見つからない
- And file backend の tokens.json には該当キーが存在する
- When `auth status` を実行する
- Then `SLACKRS_TOKEN_STORE=file` を設定する案内が表示される

### Requirement: `SLACK_TOKEN` の設定有無を表示する
`SLACK_TOKEN` が設定されている場合、`auth status` は値を表示せずに「設定済み」であることのみを表示しなければならない。(MUST)

#### Scenario: `SLACK_TOKEN` が設定されている
- Given `SLACK_TOKEN` が設定されている
- When `auth status` を実行する
- Then `SLACK_TOKEN: set` が表示される
