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

`auth login` は `--ngrok [path]` を受け付けなければならない (MUST)。

`--ngrok` が指定され、`path` が省略された場合は実行ファイル名 `ngrok`（PATH 探索）を使用しなければならない (MUST)。

`--ngrok <path>` が指定された場合は、その `path` を使用しなければならない (MUST)。

`--ngrok` が指定された場合は `ngrok http 8765` を起動し、公開 URL を抽出して `{public_url}/callback` を redirect_uri として使用しなければならない (MUST)。OAuth 完了後に ngrok プロセスを停止しなければならない (MUST)。

`--ngrok` と `--cloudflared` が同時に指定された場合はエラーにしなければならない (MUST)。

`--cloudflared` が指定されない場合、`auth login` はユーザーに redirect_uri をプロンプトして取得し、その値を OAuth フローの `redirect_uri` として使用しなければならない (MUST)。この場合、cloudflared を起動してはならない (MUST NOT)。

#### ADDED Scenario: `--ngrok`（path 省略）で ngrok を起動し redirect_uri を解決する
- Given `auth login --ngrok` を実行する
- When OAuth フローを開始する
- Then 実行ファイル名 `ngrok` を使用して `ngrok http 8765` を起動する
- And ngrok の出力から `https://*.ngrok-free.app` の公開 URL を抽出する
- And redirect_uri を `{public_url}/callback` に設定する
- And OAuth 完了後に ngrok を停止する

#### ADDED Scenario: `--ngrok <path>` 指定で ngrok を起動し redirect_uri を解決する
- Given `auth login --ngrok <path>` を実行する
- When OAuth フローを開始する
- Then 指定された `path` の ngrok を起動する
- And 公開 URL を抽出して redirect_uri に設定する

#### ADDED Scenario: `--ngrok` と `--cloudflared` の同時指定はエラーになる
- Given `auth login --ngrok --cloudflared` を実行する
- When 引数を解決する
- Then 競合エラーが表示され OAuth フローを開始しない

### Requirement: Prompt manifest installation before OAuth authentication

`auth login` は OAuth 認証を開始する前に、生成済みの Slack App Manifest をユーザーにインストールさせるための案内を表示し、Slack App 管理ページをブラウザで開いた上で、ユーザーの確認入力を待たなければならない (MUST)。

#### Scenario: OAuth 認証前に案内と確認待ちが行われる
- Given `auth login` を実行する
- When マニフェストが生成され保存される
- Then `https://api.slack.com/apps` を開く試行が行われる
- And マニフェストの保存先が表示される
- And ユーザーの確認入力を待ってから OAuth 認証フローを開始する

### Requirement: auth status displays token information
`auth status` MUST display available token types, their scopes, and the default token type. (MUST)
#### Scenario: Both User and Bot tokens exist
- Given User Token and Bot Token are saved
- When executing `auth status`
- Then both token types and their scopes are displayed

### Requirement: Display Bot ID when Bot Token exists
When a Bot Token is saved, `auth status` MUST display the Bot ID. (MUST)
#### Scenario: Bot ID is displayed for Bot Token
- Given Bot Token and Bot ID are saved
- When executing `auth status`
- Then Bot ID is displayed

