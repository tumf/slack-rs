1. [x] 既定パス算出を `slack-rs` に変更し、旧パス算出用のヘルパーを追加する
   - 検証: `src/profile/storage.rs` の既定パス算出に `slack-rs` が使われていることを確認し、`cargo test test_default_config_path` が通る
2. [x] 旧パスから新パスへの移行ロジックを実装し、既定パス取得/読み込みの経路に統合する
   - 検証: 旧パスのみ存在するケースのユニットテストを追加し、`cargo test test_migrate_legacy_config_path` が通る
3. [x] 既定パスの利用箇所が新しい挙動を通ることを確認する（CLI/auth/profile の共通経路）
   - 検証: `default_config_path` が呼ばれている箇所（`src/main.rs`, `src/cli/mod.rs`, `src/auth/commands.rs`, `src/auth/export_import.rs`）で新挙動を前提にしていることをコードレビューで確認する
4. [x] ドキュメントの設定パス記述を更新する
   - 検証: `README.md` の設定パスが `~/.config/slack-rs/profiles.json` になっていることを確認する
5. [x] 全てのテストとlintが通ることを確認する
   - 検証: `cargo test` と `cargo clippy -- -D warnings` が成功することを確認する

## Acceptance #1 Failure Follow-up
- [x] `src/profile/storage.rs` の `legacy_config_path` を `slack-cli` の旧パスに一致させ、旧パスの `profiles.json` を検出・移行できるようにする
