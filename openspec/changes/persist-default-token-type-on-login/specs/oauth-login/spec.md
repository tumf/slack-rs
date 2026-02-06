# oauth-login Delta Specification (default token type persistence)

## MODIFIED Requirements
### Requirement: Exchange authorization code for token and save
`oauth.v2.access` の成功レスポンスに含まれる `access_token` およびプロファイルに必要なメタデータは保存しなければならない (MUST)。

また、`authed_user.access_token` が存在する場合は bot トークンとは別の user トークンとして保存しなければならない (MUST)。

プロフィールの `default_token_type` が未設定の場合、取得できたトークンに基づいて `default_token_type` を保存しなければならない (MUST)。`authed_user.access_token` が存在する場合は `user` を、存在しない場合は `bot` を既定として保存する。

#### Scenario: user トークンが返る場合に default_token_type が user で保存される
- Given OAuth レスポンスに `access_token` と `authed_user.access_token` の両方が含まれる
- And 既存プロファイルの `default_token_type` が未設定である
- When トークン交換を実行する
- Then bot トークンと user トークンがそれぞれ永続化される
- And `default_token_type` が `user` として保存される
