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

`auth login` 実行時に、入力された OAuth 設定を基に Slack App Manifest を自動的に生成し、ファイルに保存しなければならない (MUST)。

このとき Manifest の `oauth_config.redirect_urls` は、redirect_uri の解決方法（cloudflared 使用有無）に整合していなければならない (MUST)。

#### Scenario: auth login 実行時に Manifest ファイルが保存される
- Given `auth login` を実行する
- When OAuth フローが完了する
- Then Manifest が `~/.config/slack-rs/<profile>_manifest.yml` に保存される
- And 保存成功時にファイルパスが表示される

#### Scenario: Manifest ファイルは Slack App 設定にアップロード可能
- Given `auth login` で生成された Manifest ファイルがある
- When ユーザーが Slack App 設定画面を開く
- Then 生成された YAML ファイルをアップロードして App 設定を更新できる

#### Scenario: cloudflared を使う場合、redirect_urls は trycloudflare のワイルドカードを含む
- Given `auth login --cloudflared <path>` を実行する
- When OAuth フローが完了し Manifest が保存される
- Then `oauth_config.redirect_urls` に `https://*.trycloudflare.com/callback` が含まれる

#### Scenario: cloudflared を使わない場合、redirect_urls はユーザー入力の redirect_uri を含む
- Given `auth login` を実行する
- And `--cloudflared` が指定されていない
- When OAuth フローが完了し Manifest が保存される
- Then `oauth_config.redirect_urls` にユーザーが入力した redirect_uri が含まれる

