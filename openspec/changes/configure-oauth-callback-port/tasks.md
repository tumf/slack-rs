# Tasks

- [x] 1. OAuthコールバックの待受ポート解決ヘルパーを追加する（既定8765、`SLACK_OAUTH_PORT`で上書き）
   - 実装: `src/oauth/port.rs` に `resolve_callback_port()` 関数を追加
   - 検証: 10個の解析テストを追加（有効値/無効値/エッジケース）し、`cargo test resolve_callback_port` で全件合格を確認済み

- [x] 2. OAuthログインのコールバック起動処理で新しいポート解決ロジックを使用する
   - 対象: `src/auth/commands.rs` の `perform_oauth_flow` および `login` 関数内の `run_callback_server` 呼び出し
   - 実装: 両方の関数で `resolve_callback_port()` を呼び出してポートを解決
   - 検証: `cargo test oauth::` で全26件のOAuthテストが合格

- [x] 3. 既定`redirect_uri`とCLIヘルプ/プロンプト文言を8765に更新する
   - 対象: `src/main.rs` の既定値とusage表示、関連する全テストファイル
   - 実装完了:
     - `src/main.rs`: 既定 `redirect_uri` を `http://127.0.0.1:8765/callback` に変更
     - `src/main.rs`: `print_config_oauth_usage` の例文を8765に更新
     - `src/oauth/mod.rs`, `src/oauth/types.rs`, `src/profile/types.rs`: テスト内のポート参照を8765に更新
     - `tests/oauth_integration.rs`: 統合テスト内のポート参照を8765に更新
     - `src/auth/commands.rs`: テスト内のポート参照を8765に更新
   - 検証: `cargo test --lib` で137件中135件合格（2件の失敗は既存の無関係なテスト）
