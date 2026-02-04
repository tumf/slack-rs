# Tasks

- [x] 1. プリセット `all` のスコープ一覧をコードに定義し、安定順序で返すヘルパーを追加する
   - 検証: `src/oauth/scopes.rs` に `all_scopes()` と `expand_scopes()` を実装し、単体テストで確認済み（`cargo test oauth::scopes`）

- [x] 2. スコープ入力の解析を更新し、`all` を含む場合にプリセット展開と重複除去を行う
   - 対象: 対話入力（`auth login`）と `config oauth set --scopes`
   - 実装: `src/auth/commands.rs` の `prompt_for_scopes` と `src/commands/config.rs` の `oauth_set` で `expand_scopes` を呼び出し
   - 検証: 統合テスト（`tests/scope_preset_integration.rs`）で `all` 単独/混在/大小文字をカバー

- [x] 3. ヘルプとプロンプト文言を更新し、`all` が利用可能であることを明示する
   - 対象: CLI usage と対話プロンプト
   - 実装: `src/main.rs` の `print_config_oauth_usage` に説明を追加、`prompt_for_scopes` のプロンプトを更新
   - 検証: `src/main.rs` と `src/auth/commands.rs` に `all` への言及を確認済み
