# Tasks

1. OAuthコールバックの待受ポート解決ヘルパーを追加する（既定8765、`SLACK_OAUTH_PORT`で上書き）
   - 検証: 有効値/無効値の解析テストを追加し、`cargo test oauth_callback_port` で確認する

2. OAuthログインのコールバック起動処理で新しいポート解決ロジックを使用する
   - 対象: `src/auth/commands.rs` の `run_callback_server` 呼び出し
   - 検証: ログ出力またはユニットテストで待受ポートが反映されることを確認する

3. 既定`redirect_uri`とCLIヘルプ/プロンプト文言を8765に更新する
   - 対象: `src/main.rs` の既定値とusage表示、関連するテスト
   - 検証: `cargo test` と該当文字列の更新を確認する
