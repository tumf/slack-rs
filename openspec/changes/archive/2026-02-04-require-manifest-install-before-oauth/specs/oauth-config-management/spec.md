# oauth-config-management Specification

## MODIFIED Requirements
### Requirement: Generate Manifest automatically during auth login

`auth login` 実行時に、入力された OAuth 設定を基に Slack App Manifest を自動的に生成し、OAuth フロー開始前に一時ファイルへ保存しなければならない (MUST)。

このとき Manifest の `oauth_config.redirect_urls` は、redirect_uri の解決方法（cloudflared または ngrok 使用有無）に整合していなければならない (MUST)。

#### 保存先
- Manifest は永続的な設定ファイルとして保存してはならない (MUST)。インストール時に利用する一時ファイルとして保存する (MUST)。
- 一時ファイルの保存先は OS の一時ディレクトリ配下とし、ファイル名には識別子（例: profile_name）を含める (MUST)。
- CLI は一時ファイルのパスをユーザーに表示しなければならない (MUST)。
- OAuth フロー開始後（または終了後）に、一時ファイルの削除を試行しなければならない (MUST)。削除に失敗しても OAuth の結果自体は妨げない (MUST)。

#### リダイレクトURL整合
- `--cloudflared` の場合、`oauth_config.redirect_urls` に `https://*.trycloudflare.com/callback` と解決済み redirect_uri を含める (MUST)。
- `--ngrok` の場合、`oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` と解決済み redirect_uri を含める (MUST)。
- それ以外の場合、`oauth_config.redirect_urls` は解決済み redirect_uri のみを含める (MUST)。

#### エラーハンドリング
- Manifest の生成または保存に失敗した場合、OAuth フローを開始してはならない (MUST)。

#### Scenario: OAuth 開始前に Manifest が保存される
- Given `auth login --ngrok` を実行する
- When OAuth フローを開始する前に Manifest を生成する
- Then Manifest がファイルに保存される
- And `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` が含まれる
