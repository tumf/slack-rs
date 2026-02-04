# oauth-manifest-generation Specification

## Purpose
TBD - created by archiving change generate-manifest-from-profile. Update Purpose after archive.
## Requirements
### Requirement: Generate Slack App Manifest automatically during auth login execution

`auth login` 実行時に、ユーザーが入力した `client_id`、`bot_scopes`、`user_scopes`、および解決された `redirect_uri`（cloudflared、ngrok、またはプロンプト入力）を使用して、Slack App Manifest の YAML を自動的に生成しなければならない (MUST)。

#### ADDED Scenario: ngrok 使用時は ngrok のワイルドカード URL が含まれる
- Given `auth login --ngrok` を実行する
- When Manifest が生成される
- Then `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる

### Requirement: Manifest generation does not depend on external APIs

Manifest の生成はローカルの設定値のみで完結し、Slack API への問い合わせを行ってはならない (MUST NOT)。

#### Scenario: 生成時に外部呼び出しがない
- Given Manifest を生成する
- When 生成処理を実行する
- Then ネットワークアクセスが発生しない

### Requirement: Copy manifest to clipboard after generation

After the Manifest YAML is generated and saved during `auth login` execution, the YAML MUST be copied to the OS clipboard.
If the clipboard operation fails, a warning MUST be displayed and the process MUST NOT be interrupted.

#### Scenario: Clipboard is copied when available
- Given executing `auth login` and generating Manifest YAML
- When the manifest is saved to a file
- Then the same YAML is copied to the clipboard

#### Scenario: Continue with warning when clipboard is unavailable
- Given executing `auth login` in an environment where clipboard operations fail
- When attempting to copy to clipboard after saving the manifest
- Then a warning is displayed and the login process continues

