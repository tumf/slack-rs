# oauth-config-management Specification

## Purpose
TBD - created by archiving change add-oauth-config-management. Update Purpose after archive.
## Requirements
### Requirement: OAuth configuration can be managed via CLI

OAuth configuration per profile MUST be settable, viewable, and deletable via CLI.

#### Scenario: set/show/delete commands are provided
- `slackrs config oauth set --profile <name>` can save configuration
- `slackrs config oauth show --profile <name>` can view saved content
- `slackrs config oauth delete --profile <name>` can delete configuration

### Requirement: client_secret is stored in Keyring and not displayed

`client_secret` は設定ファイルに保存してはならず (MUST NOT)、token store backend に保存されなければならない (MUST)。

token store backend のデフォルトは Keyring でなければならない (MUST)。

`config oauth show` は backend に関わらず `client_secret` を出力してはならない (MUST NOT)。

#### Scenario: show は client_secret を表示しない
- Given `config oauth set` で `client_secret` を入力する
- When `config oauth show` を実行する
- Then 出力に `client_secret` の値が含まれない
- And `profiles.json` に `client_secret` が含まれない

#### Scenario: file mode でも client_secret は表示されない
- Given `SLACKRS_TOKEN_STORE=file` が設定されている
- And `client_secret` が file backend に保存されている
- When `config oauth show` を実行する
- Then 出力に `client_secret` の値が含まれない

### Requirement: Generate Manifest automatically during auth login

`auth login` 実行時に、入力された OAuth 設定を基に Slack App Manifest を自動的に生成し、OAuth フロー開始前にファイルへ保存しなければならない (MUST)。

このとき Manifest の `oauth_config.redirect_urls` は、redirect_uri の解決方法（cloudflared または ngrok 使用有無）に整合していなければならない (MUST)。

#### Scenario: OAuth 開始前に Manifest が保存される
- Given `auth login --ngrok` を実行する
- When OAuth フローを開始する前に Manifest を生成する
- Then Manifest がファイルに保存される
- And `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる

### Requirement: Provide command to configure default token type
The `config` subcommand MUST allow setting the profile's `default_token_type`. (MUST)
#### Scenario: Execute `config set default --token-type user`
- Given target profile exists
- When executing `config set default --token-type user`
- Then `default_token_type=user` is saved to profile

### Requirement: Keyring unavailable must fail without silent fallback

`SLACKRS_TOKEN_STORE` による明示指定がない場合、Keyring が利用不能な環境で `config oauth set/show/delete` を実行してはならない (MUST NOT)。その場合は MUST で失敗し、対処方法（例: OS の Keyring をアンロックして再実行する、または `SLACKRS_TOKEN_STORE=file` を設定する）を提示しなければならない。

ここで「Keyring が利用不能」には、Keyring がロックされていてユーザー操作（対話的アンロック）を要求するケース（interaction required 等）も含む。slack-rs は Keyring backend のアンロックのために独自のパスワード/パスフレーズ入力プロンプトを実装してはならない (MUST NOT)。

#### Scenario: Keyring 利用不能時に set が失敗しガイダンスを出す
- Given `SLACKRS_TOKEN_STORE` が設定されていない
- And Keyring が利用不能である
- When `config oauth set` を実行する
- Then コマンドは失敗する
- And エラーに「OS の Keyring をアンロックして再実行」または `SLACKRS_TOKEN_STORE=file` を含む対処方法が含まれる
- And slack-rs は Keyring アンロックのための追加のパスワード入力を求めない

### Requirement: client_secret can be obtained securely

`config oauth set` MUST be able to obtain `client_secret` via the following input sources:
1) Environment variable specified with `--client-secret-env <VAR>`
2) `SLACKRS_CLIENT_SECRET`
3) `--client-secret-file <PATH>`
4) `--client-secret <SECRET>` (requires explicit consent via `--yes`)
5) Interactive input (if none of the above are provided)

If `--client-secret` is specified without `--yes`, it MUST be rejected for security reasons and alternative methods MUST be suggested.

#### Scenario: Obtain client_secret from environment variable
- Given `SLACKRS_CLIENT_SECRET` is set
- When `config oauth set <profile> --client-id ... --redirect-uri ... --scopes ...` is executed
- Then `client_secret` is obtained without interactive input
- And `client_secret` is stored in the token store backend

#### Scenario: `--client-secret` requires `--yes`
- Given `--client-secret` is specified
- And `--yes` is not specified
- When `config oauth set` is executed
- Then the command MUST fail
- And alternative methods (environment variable/file/interactive input) MUST be suggested

