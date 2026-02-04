## 設計方針

### スコープの扱い
- OAuth v2 の `scope` と `user_scope` を分離して扱う。
- 内部表現は `bot_scopes` / `user_scopes` を基本とし、旧 `scopes` は `bot_scopes` にマッピングして後方互換を保つ。

### 入力解決の優先順位

#### auth login のスコープ入力
- `auth login` は bot/user のスコープを対話的に入力する。
- ただし、明示的な CLI 引数が指定されている場合は CLI 引数を優先し、該当プロンプトは表示しない。
- 推奨プロンプト（2 回）
  - Bot scopes: 入力は `all` がデフォルト
  - User scopes: 入力は `all` がデフォルト

#### 入力形式
- 入力はカンマ区切り（例: `channels:read,chat:write`）またはプリセット名を受け付ける。
- プリセット `all` はプロンプトの種類に応じて展開する（bot プロンプトでは `bot:all`、user プロンプトでは `user:all`）。
- `bot:all` と `user:all` は明示プリセットとして受け付ける。
- 展開後は bot/user を完全に分離して保持する（OAuth URL の `scope` と `user_scope` を混ぜない）。

### cloudflared tunnel による動的 redirect_uri 生成

#### 概要
cloudflared は OPTIONAL とし、`auth login` は次の 2 つの redirect_uri 解決方法を提供する。

- **cloudflared を使う場合**: `--cloudflared [path]` が指定されたとき、cloudflared tunnel を起動し、生成された公開 URL を redirect_uri として使用する。
  - `path` が省略された場合、実行ファイル名 `cloudflared`（PATH から探索）を使用する。
  - `path` が指定された場合、指定されたパスを実行ファイルとして使用する。
- **cloudflared を使わない場合**: `--cloudflared` が指定されないとき、ユーザーに redirect_uri をプロンプトして決定する。

#### 処理フロー
`--cloudflared [path]` が指定された場合:

1. **実行ファイル解決**: `path` が指定されていればその `path` を使用し、未指定であれば `cloudflared` を使用する
2. **Tunnel 起動**: 解決した cloudflared を用いて `tunnel --url http://localhost:8765` を実行
3. **URL 抽出**: cloudflared の標準出力から公開 URL（例: `https://xxx.trycloudflare.com`）を抽出
4. **redirect_uri 設定**: 抽出した URL に `/callback` を付加して redirect_uri とする（例: `https://xxx.trycloudflare.com/callback`）
5. **OAuth フロー実行**: 生成した redirect_uri を使用して OAuth 認証を実行
6. **Tunnel 停止**: OAuth フロー完了後、cloudflared プロセスを終了

`--cloudflared` が指定されない場合:

1. **redirect_uri 入力**: ユーザーに redirect_uri をプロンプトして取得
2. **OAuth フロー実行**: 入力された redirect_uri を使用して OAuth 認証を実行

#### 技術的詳細
- cloudflared の出力パターン: `https://[random-subdomain].trycloudflare.com` の形式で URL が出力される
- URL 抽出は正規表現 `https://[a-zA-Z0-9-]+\.trycloudflare\.com` を使用
- Tunnel プロセスは OAuth コールバック受信後に即座に終了させる
- エラーハンドリング: cloudflared が起動しない場合は明確なエラーメッセージを表示

#### Slack App 設定要件
- cloudflared を使う場合、Slack App の Redirect URLs に `https://*.trycloudflare.com/callback` を追加する必要がある
- ワイルドカード URL により、毎回異なる tunnel URL でも認証可能

### auth login 時の Manifest 自動生成

#### 概要
`auth login` 実行時に、ユーザーが入力した `client_id`、`bot_scopes`、`user_scopes`、および解決された redirect_uri（cloudflared またはプロンプト入力）を使用して、Slack App Manifest を自動的に生成する。

#### 処理フロー
1. **ユーザー入力**: `client_id`、`bot_scopes`、`user_scopes` を CLI 引数または対話入力で取得
2. **redirect_uri 解決**: `--cloudflared` が指定されていれば tunnel から生成し、未指定ならプロンプト入力で取得
3. **OAuth フロー実行**: 解決した redirect_uri を使用して OAuth 認証を実行
4. **Manifest 生成**: OAuth フロー完了後、入力情報と redirect_uri から Manifest YAML を生成
5. **ファイル保存**: 生成した Manifest を `~/.config/slack-rs/<profile>_manifest.yml` に保存

#### Manifest の内容
- redirect_urls は redirect_uri の解決方法に応じて反映する
  - cloudflared を使う場合: `oauth_config.redirect_urls` に `https://*.trycloudflare.com/callback` を含める
  - cloudflared を使わない場合: `oauth_config.redirect_urls` にユーザーが入力した redirect_uri を含める
- `oauth_config.scopes.bot`: `bot_scopes` の内容
- `oauth_config.scopes.user`: `user_scopes` の内容
- `settings.org_deploy_enabled`: false（デフォルト）
- `features.bot_user.display_name`: プロファイル名を使用

#### ファイル保存先
- デフォルト: `~/.config/slack-rs/<profile>_manifest.yml`
- ユーザーは生成された Manifest ファイルを Slack App 設定画面にアップロードできる
- Slack App への自動反映（API 呼び出し）は本変更の対象外

#### エラーハンドリング
- Manifest 生成に失敗した場合でも OAuth フローは完了する
- 生成失敗時は警告メッセージを表示し、ユーザーに手動設定を促す

### プリセット
- 公式プリセットは `bot:all` / `user:all` とする。
- `all` は利便性のために受け付け、入力コンテキストに応じて展開する。
  - bot スコープ入力における `all` は `bot:all` と同義
  - user スコープ入力における `all` は `user:all` と同義
- 後方互換として、旧来の単一スコープ入力（`scopes`）で `all` が指定された場合は `bot:all` と同義にする。

### テスト方針
- OAuth URL 生成に `user_scope` が付与されることをユニットテストで検証する。
- cloudflared を使う場合は tunnel の起動・URL 抽出・停止をモックして検証する。
- cloudflared を使わない場合は redirect_uri のプロンプト入力をモックして検証する。
- `auth login` 実行後に Manifest ファイルが正しく生成されることを統合テストで検証する。
- Manifest 生成は固定入力に対して安定した YAML を生成することをユニットテストで検証する。
