# oauth-manifest-generation Specification

## Purpose
TBD - created by archiving change generate-manifest-from-profile. Update Purpose after archive.
## Requirements
### Requirement: Generate Slack App Manifest automatically during auth login execution

`auth login` 実行時に、ユーザーが入力した `client_id`、`bot_scopes`、`user_scopes`、および解決された `redirect_uri`（cloudflared またはプロンプト入力）を使用して、Slack App Manifest の YAML を自動的に生成しなければならない (MUST)。

#### Scenario: auth login 実行時に Manifest が生成される
- Given `auth login` を実行する
- And ユーザーが `client_id`、`bot_scopes`、`user_scopes` を入力する
- And redirect_uri が解決される
- When OAuth フローが完了する
- Then Manifest YAML が `~/.config/slack-rs/<profile>_manifest.yml` に保存される
- And YAML に `oauth_config.redirect_urls` と `scopes.bot` と `scopes.user` が含まれる

#### Scenario: cloudflared を使う場合、Manifest は cloudflared のワイルドカード URL を含める
- Given `auth login --cloudflared <path>` で Manifest を生成する
- When 生成された YAML を確認する
- Then `oauth_config.redirect_urls` に `https://*.trycloudflare.com/callback` が含まれる
- And これにより、毎回異なる tunnel URL でも OAuth 認証が可能になる

#### Scenario: cloudflared を使わない場合、Manifest はユーザー入力の redirect_uri を含める
- Given `auth login` で Manifest を生成する
- And `--cloudflared` が指定されていない
- And redirect_uri をユーザーが入力している
- When 生成された YAML を確認する
- Then `oauth_config.redirect_urls` にユーザーが入力した redirect_uri が含まれる

#### Scenario: Manifest 生成失敗時も OAuth フローは完了する
- Given `auth login` を実行する
- And Manifest 生成処理でエラーが発生する
- When OAuth フローを実行する
- Then OAuth 認証は正常に完了する
- And 警告メッセージが表示される
- And ユーザーに手動設定を促すメッセージが表示される

### Requirement: Manifest generation does not depend on external APIs

Manifest の生成はローカルの設定値のみで完結し、Slack API への問い合わせを行ってはならない (MUST NOT)。

#### Scenario: 生成時に外部呼び出しがない
- Given Manifest を生成する
- When 生成処理を実行する
- Then ネットワークアクセスが発生しない

