- [x] 1. OAuth 設定のデータ構造を `bot_scopes`/`user_scopes` に分割し、旧 `scopes` を読み込む後方互換を追加する。
   - 検証: `profiles.json` の旧形式を読み込むテストで `bot_scopes` に移譲されることを確認する。

- [x] 2. `auth login` で bot/user スコープを対話的に入力できるようにする（デフォルト入力値は両方とも `all`）。
   - ただし、明示的な CLI 引数（例: `--bot-scopes` / `--user-scopes`）が指定されている場合は、該当スコープのプロンプトは表示しない。
   - 受け付ける入力はカンマ区切りのスコープ名またはプリセット（`all` / `bot:all` / `user:all`）とし、プロンプトの種類に応じて `all` を展開する。
   - 検証（モックファースト）:
     - 入出力モックで 2 回のプロンプトが発生し、両方ともデフォルトが `all` であることを確認する。
     - 入力が空（Enter のみ）の場合に `all` が選択されることを確認する。
     - CLI 引数指定時にプロンプトが発生しないことを確認する。

- [x] 3. OAuth 認可 URL に `user_scope` を追加し、`bot_scopes`/`user_scopes` を分離して送信する。
   - 検証: URL 生成テストで `user_scope` が含まれることを確認する。

- [x] 4. cloudflared tunnel の起動・停止機能を実装する。
   - `--cloudflared [path]` が指定された場合に cloudflared をサブプロセスとして起動（cloudflared は OPTIONAL）
     - `--cloudflared` だけが指定された場合は実行ファイル名 `cloudflared`（PATH から探索）を使用する
     - `--cloudflared <path>` が指定された場合はその `path` を使用する
   - `--cloudflared` が未指定の場合は本タスクの機能は使用しない
   - プロセスの標準出力から公開 URL を抽出する機能を実装
   - OAuth フロー完了後に cloudflared プロセスを終了する機能を実装
   - 検証: モックプロセスを使用して、URL 抽出と終了処理が正しく動作することを確認する。

- [x] 5. `auth login` に redirect_uri 解決の分岐を追加する（cloudflared は OPTIONAL）。
   - `--cloudflared [path]` が指定されている場合:
      - Tunnel 起動時に公開 URL を抽出し、`/callback` を付加して redirect_uri とする
      - 生成した redirect_uri を OAuth フローで使用する
      - OAuth コールバック受信後に tunnel を停止する
      - エラーハンドリング: 指定 `path` の cloudflared が実行できない場合、または `cloudflared`（PATH 解決）が実行できない場合や起動失敗時の適切なエラーメッセージ
   - `--cloudflared` が指定されていない場合:
      - redirect_uri をユーザーにプロンプトして取得し、OAuth フローで使用する
      - cloudflared を起動しない
   - 検証（モックファースト、実 Slack 資格情報不要）:
     - 既存の `base_url` 上書きを使用して OAuth エンドポイント（認可コード交換など）をモックサーバに向ける。
     - cloudflared ありのケースはモックプロセス出力（例: `https://abc.trycloudflare.com`）で代替し、実プロセス起動を不要にする。
     - cloudflared なしのケースは入出力モックで redirect_uri のプロンプトが発生することを確認する。
     - ブラウザ操作や Slack UI を前提にせず、ローカルのコールバック受信はテストから HTTP リクエストで疑似的に完了させる。

- [x] 6. Manifest 生成機能を実装する。
   - `client_id`、`bot_scopes`、`user_scopes`、`redirect_uri` から Slack App Manifest YAML を生成する関数を実装
   - redirect_uri の解決方法に応じて `oauth_config.redirect_urls` を切り替える
     - cloudflared を使う場合: `https://*.trycloudflare.com/callback` を含める
     - cloudflared を使わない場合: ユーザーが入力した redirect_uri を含める
   - 検証: 固定入力に対して期待する YAML が生成されるユニットテストを追加する。

- [x] 7. `auth login` に Manifest 自動生成機能を統合する。
   - OAuth フロー完了後、入力情報から Manifest を生成
   - 生成した Manifest を `~/.config/slack-rs/<profile>_manifest.yml` に保存
   - 保存成功時にファイルパスをユーザーに通知
   - 生成失敗時は警告メッセージを表示（OAuth フローは継続）
    - 検証（モックファースト、実 Slack 資格情報不要）:
      - `auth login` のテストで、プロンプト/CLI 引数で選択された bot/user スコープが Manifest の `scopes.bot` / `scopes.user` に反映されることを確認する。
      - cloudflared ありのケースで `oauth_config.redirect_urls` に `https://*.trycloudflare.com/callback` が含まれることを確認する。
      - cloudflared なしのケースで `oauth_config.redirect_urls` にユーザー入力の redirect_uri が含まれることを確認する。
      - 実 Slack UI でのアップロード確認など、人手を必要とする検証は行わない。

- [x] 8. CLI ドキュメント/ヘルプに Manifest 自動生成の説明を追加する。
   - cloudflared tunnel の使用方法と Slack App 設定要件（`https://*.trycloudflare.com/callback`）を記載
   - Manifest ファイルの保存先と使用方法を説明
   - 検証: `--help` 出力に該当説明が含まれることを確認する。

## Acceptance #1 Failure Follow-up
- [x] `--bot-scopes` / `--user-scopes` の CLI 入力でも `all` / `bot:all` / `user:all` をコンテキストに応じて展開する（`src/main.rs` の解析結果が `src/auth/commands.rs` に渡る前に展開）。
- [x] OAuth 応答の bot トークンと user トークンを別キーで保存できるようにし、`authed_user.access_token` がある場合は両方を永続化する（`src/auth/commands.rs` の OAuth フローと保存処理を更新）。
- [x] `--cloudflared` 未指定時は、保存済み `redirect_uri` があっても必ずプロンプトで取得する（`src/auth/commands.rs` の redirect_uri 解決を修正）。

## Acceptance #2 Failure Follow-up
- [x] bot トークンの Keyring 保存キーを `team_id:user_id`（`make_token_key`）に戻し、user トークンは別キーで保存する（`src/auth/commands.rs` の `save_profile_and_credentials` を修正）。
