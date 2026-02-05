## MODIFIED Requirements
### Requirement: auth status displays token information
`auth status` は `default_token_type` がプロフィールに設定されている場合はその値を表示し、未設定の場合のみ従来の推測ロジック（User Token があれば User、なければ Bot）を使用しなければならない。(MUST)

#### Scenario: default_token_type が user のときに表示される
- Given User Token と Bot Token が保存されている
- And プロフィールの `default_token_type` が `user` に設定されている
- When `auth status` を実行する
- Then `Default Token Type: User` が表示される
