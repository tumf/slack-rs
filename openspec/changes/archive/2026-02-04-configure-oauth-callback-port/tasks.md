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

## Acceptance #1 Failure Follow-up
- [x] `SLACK_OAUTH_PORT` が空文字や空白のみのときに既定8765へフォールバックせず、設定エラーとして拒否する（実装: `src/oauth/port.rs` の `resolve_callback_port`、関連テスト更新）
   - 実装: 空文字および空白のみの環境変数を `ConfigError` として拒否
   - 検証: `test_resolve_callback_port_empty_env` および `test_resolve_callback_port_whitespace_only` テストを更新し、全11件のポート解析テストが合格
   - コード品質: `cargo fmt` および `cargo clippy -- -D warnings` で問題なし

## 実装完了サマリー

### 変更内容
- **新規ファイル**: `src/oauth/port.rs` - ポート解決ロジックとテスト
- **更新ファイル**:
  - `src/auth/commands.rs`: `login_with_credentials` および `perform_oauth_flow` で `resolve_callback_port()` を使用
  - `src/oauth/mod.rs`: `resolve_callback_port` を公開エクスポート
  - `src/main.rs`: 既定 `redirect_uri` を `http://127.0.0.1:8765/callback` に変更
  - 各種テストファイル: ポート参照を8765に更新

### テスト結果
- ポート解決テスト: 11件全て合格
- OAuth関連テスト: 27件全て合格
- ライブラリテスト全体: 138件中136件合格（2件の失敗は既存の無関係なテスト）
- コード品質: `cargo fmt --check` および `cargo clippy -- -D warnings` で問題なし

### 受け入れ基準の達成
✓ `SLACK_OAUTH_PORT` が未設定の場合、既定ポートは8765である
✓ `SLACK_OAUTH_PORT` が有効な数値ならそのポートで待受する
✓ 無効な`SLACK_OAUTH_PORT`は起動前にエラーとして扱われる
✓ 空文字や空白のみの`SLACK_OAUTH_PORT`は設定エラーとして拒否される
✓ 既定の`redirect_uri`が8765に更新される
✓ CLIヘルプテキストが8765を示す
