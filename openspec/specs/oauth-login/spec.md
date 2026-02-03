# oauth-login Specification

## Purpose
TBD - created by archiving change add-oauth-auth-flow. Update Purpose after archive.
## Requirements
### Requirement: PKCE と state を用いた認証 URL を生成できる
OAuth 認可 URL には `client_id`, `redirect_uri`, `state`, `code_challenge`, `code_challenge_method=S256` を含めなければならない。(MUST)
#### Scenario: 認証 URL に必須パラメータが含まれる
- Given OAuth 設定が読み込まれている
- When 認証 URL を生成する
- Then 必須パラメータがすべて含まれる

### Requirement: 必須設定が未設定の場合は開始しない
`SLACKCLI_CLIENT_ID` または `SLACKCLI_CLIENT_SECRET` が未設定の場合、OAuth フローを開始してはならない。(MUST NOT)
#### Scenario: 必須環境変数が不足している
- Given `SLACKCLI_CLIENT_ID` が未設定である
- When `auth login` を実行する
- Then エラーで終了する

### Requirement: localhost callback で state を検証する
callback の `state` が一致しない場合、認可コードを受理してはならない。(MUST NOT)
#### Scenario: state が一致しない
- Given callback サーバが起動している
- When `code` と不一致の `state` が送られる
- Then エラーになる

### Requirement: callback の受信にはタイムアウトがある
callback を一定時間以内に受信できなければならない。(MUST)
#### Scenario: タイムアウトまで code を受信しない
- Given callback サーバが起動している
- When 指定時間内に code が届かない
- Then タイムアウトエラーになる

### Requirement: 認可コードを token に交換し保存する
`oauth.v2.access` の成功応答から `access_token` と profile メタデータを保存しなければならない。(MUST)
#### Scenario: `oauth.v2.access` の成功応答を保存する
- Given 有効な code が存在する
- When token 交換を実行する
- Then access_token と profile メタデータが保存される

### Requirement: 同一 `(team_id, user_id)` は更新として扱う
同一の `(team_id, user_id)` が存在する場合、新規 profile を追加せず既存の token/メタ情報を更新しなければならない。(MUST)
#### Scenario: 既存のアカウントで再ログインする
- Given 同じ `(team_id, user_id)` を持つ profile が存在する
- When `auth login` を実行する
- Then 既存 profile が更新される

### Requirement: auth コマンドでプロファイル操作ができる
`auth status/list/rename/logout` は profile の参照・更新・削除を実行できなければならない。(MUST)
#### Scenario: auth list が profiles.json の内容を返す
- Given 複数の profile が保存されている
- When `auth list` を実行する
- Then profile 一覧が返る

