# oauth-login Specification

## Purpose
Defines the OAuth 2.0 PKCE authentication flow for slack-rs, enabling secure login to Slack workspaces without exposing client secrets.
## Requirements
### Requirement: Generate authentication URL with PKCE and state

OAuth 認可 URL には `client_id`、`redirect_uri`、`state`、`code_challenge`、`code_challenge_method=S256` を含めなければならない (MUST)。

さらに、user スコープが 1 つ以上ある場合は `user_scope` も含めなければならない (MUST)。

#### Scenario: user スコープがある場合に user_scope が付与される
- Given OAuth 設定に `user_scopes` が存在する
- When 認可 URL を生成する
- Then URL に `user_scope` が含まれる

#### Scenario: user スコープが空の場合は user_scope を付与しない
- Given `user_scopes` が空である
- When 認可 URL を生成する
- Then URL に `user_scope` を含めない

### Requirement: Do not start if required configuration is missing

login 開始時に OAuth クライアント情報が不足している場合、対話入力で補完しなければならない (MUST)。

スコープについては、明示的な CLI 引数が指定されている場合を除き、対話的に入力しなければならない (MUST)。
対話入力のデフォルト入力値は bot/user ともに `all` でなければならない (MUST)。

#### Scenario: スコープが CLI 引数で指定されていない場合は対話入力される
- Given `--bot-scopes` と `--user-scopes` が指定されていない
- When `auth login` を実行する
- Then bot スコープの入力プロンプトが表示される
- And user スコープの入力プロンプトが表示される
- And いずれのデフォルト入力値も `all` である

#### Scenario: スコープが CLI 引数で指定されている場合は対話入力しない
- Given `--bot-scopes` または `--user-scopes` が指定されている
- When `auth login` を実行する
- Then 指定されている側のスコープについて入力プロンプトを表示しない

### Requirement: Validate state in localhost callback
The authorization code MUST NOT be accepted if the callback `state` does not match. (MUST NOT)
#### Scenario: State does not match
- Given callback server is running
- When `code` is sent with mismatched `state`
- Then an error occurs

### Requirement: Callback reception has a timeout
The callback MUST be received within a certain time period. (MUST)
#### Scenario: Code is not received before timeout
- Given callback server is running
- When code does not arrive within the specified time
- Then a timeout error occurs

### Requirement: Exchange authorization code for token and save

`oauth.v2.access` の成功レスポンスに含まれる `access_token` およびプロファイルに必要なメタデータは保存しなければならない (MUST)。

また、`authed_user.access_token` が存在する場合は bot トークンとは別の user トークンとして保存しなければならない (MUST)。

#### Scenario: bot/user 両方のトークンが返る場合に別々に保存される
- Given OAuth レスポンスに `access_token` と `authed_user.access_token` の両方が含まれる
- When トークン交換を実行する
- Then bot トークンと user トークンがそれぞれ永続化され、独立して取得できる

### Requirement: Same `(team_id, user_id)` is treated as update
When the same `(team_id, user_id)` exists, existing token/metadata MUST be updated instead of adding a new profile. (MUST)
#### Scenario: Re-login with existing account
- Given a profile with the same `(team_id, user_id)` exists
- When executing `auth login`
- Then the existing profile is updated

### Requirement: Auth commands can manipulate profiles
`auth status/list/rename/logout` MUST be able to read, update, and delete profiles. (MUST)
#### Scenario: auth list returns profiles.json content
- Given multiple profiles are saved
- When executing `auth list`
- Then a list of profiles is returned

### Requirement: Do not use environment variables for OAuth configuration resolution

OAuth configuration MUST be resolved from CLI arguments or profile configuration files, and MUST NOT reference environment variables.

#### Scenario: Environment variables are ignored even when set
- Environment variables such as `SLACKRS_CLIENT_ID` are set
- `client_id` exists in profile configuration
- When `slack-rs login` is executed, environment variables are not referenced and profile configuration is used

### Requirement: Resolve redirect_uri (cloudflared is OPTIONAL)

`auth login` は `--cloudflared [path]` オプションで cloudflared 実行ファイルを受け付けなければならない (MUST)。

`--cloudflared` が存在し `path` が省略された場合、`auth login` は実行ファイル名 `cloudflared`（PATH から探索）を使用しなければならない (MUST)。

`--cloudflared <path>` が存在する場合、`auth login` は指定された `path` を使用しなければならない (MUST)。

`--cloudflared` が指定された場合、`auth login` は cloudflared tunnel プロセスを起動し、生成された公開 URL を抽出し、それを OAuth フローの `redirect_uri` として使用しなければならない (MUST)。tunnel は OAuth フロー完了後に停止しなければならない (MUST)。

`--cloudflared` が指定されない場合、`auth login` はユーザーに redirect_uri をプロンプトして取得し、その値を OAuth フローの `redirect_uri` として使用しなければならない (MUST)。この場合、cloudflared を起動してはならない (MUST NOT)。

#### Scenario: `--cloudflared <path>` 指定時に tunnel を起動し公開 URL を redirect_uri に使用する
- Given `auth login --cloudflared <path>` を実行する
- When OAuth フローを開始する
- Then 指定された cloudflared を `tunnel --url http://localhost:8765` で起動する
- And cloudflared の出力から公開 URL（例: `https://xxx.trycloudflare.com`）を抽出する
- And redirect_uri を `{public_url}/callback` に設定する
- And OAuth コールバック受信後に tunnel を停止する

#### Scenario: `--cloudflared`（path 省略）指定時にデフォルト実行ファイル名で tunnel を起動する
- Given `auth login --cloudflared` を実行する
- When OAuth フローを開始する
- Then cloudflared 実行ファイルとして `cloudflared` を使用する
- And `tunnel --url http://localhost:8765` で起動する
- And cloudflared の出力から公開 URL（例: `https://xxx.trycloudflare.com`）を抽出する
- And redirect_uri を `{public_url}/callback` に設定する
- And OAuth コールバック受信後に tunnel を停止する

#### Scenario: `--cloudflared` 未指定時は redirect_uri をプロンプトして cloudflared を起動しない
- Given `auth login` を実行する
- And `--cloudflared` が指定されていない
- When OAuth フローを開始する
- Then redirect_uri の入力プロンプトが表示される
- And 入力された redirect_uri を使用する
- And cloudflared tunnel を起動しない

#### Scenario: `--cloudflared` 指定時に cloudflared が実行できない場合のエラーハンドリング
- Given `auth login --cloudflared [path]` を実行する
- And cloudflared 実行ファイルが未存在、または実行できない
- When OAuth フローを開始する
- Then cloudflared が実行できないことが分かる明確なエラーメッセージを表示する
- And OAuth フローを開始しない

