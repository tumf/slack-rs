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

`client_secret` MUST NOT be saved to configuration files but stored in Keyring, and MUST NOT be output by the `show` command.

#### Scenario: Secret is not output
- Input `client_secret` when executing `config oauth set`
- `profiles.json` does not contain `client_secret`
- `config oauth show` output does not contain `client_secret`

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

