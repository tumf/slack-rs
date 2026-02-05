- [x] グローバルな `--non-interactive` フラグを追加し、CLI の引数解析から各コマンドに伝搬する（確認: `tests/non_interactive_integration.rs` でフラグが認識されるテストを追加し `cargo test non_interactive_integration` が通る）
- [x] 破壊操作の確認ガードに non-interactive 判定を追加し、`--yes` が無い場合は即時エラーで終了する（確認: `src/commands/guards.rs` のユニットテストで非対話時にプロンプトが走らずエラーになることを検証する）
- [x] `auth login` の不足入力プロンプトを非対話モードで抑止し、不足項目と指定方法を列挙するエラーに切り替える（確認: `src/auth/commands.rs` のテストで `--non-interactive` 時に入力待ちせずエラーになる）
- [x] TTY 自動判定のヘルパーを導入し、stdin 非 TTY では non-interactive を暗黙有効にする（確認: 判定関数をテスト可能な形に分離し、テストで TTY なし相当の挙動を検証する）
- [x] ヘルプ/usage に `--non-interactive` の説明を追加する（確認: `src/main.rs` のヘルプ出力にフラグ説明が含まれる）

## Acceptance #1 Failure Follow-up
- [x] 作業ツリーがクリーンではないため、再検証前に未コミット変更を整理する（`openspec/changes/define-agentic-cli-contract/tasks.md` が変更中）
- [x] `msg delete` の非対話失敗時に終了コード 2 と再実行例が出るようにする（`src/commands/guards.rs` の `confirm_destructive` と `src/main.rs` の終了コード処理）
- [x] `auth login --non-interactive` の不足項目をまとめて列挙し、各項目の指定方法（`--client-id`/`--bot-scopes`/`--user-scopes` など）を同時に提示する（`src/auth/commands.rs` の `login_with_credentials`）
- [x] `--non-interactive` でも `--cloudflared`/`--ngrok` ルートで stdin を読まないようにする（`src/main.rs` の `run_auth_login` と `src/auth/commands.rs` の `login_with_credentials_extended`）
