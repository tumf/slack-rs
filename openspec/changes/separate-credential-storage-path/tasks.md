- [x] `FileTokenStore::default_path()` の仕様を新デフォルト `~/.local/share/slack-rs/tokens.json` に更新し、`SLACK_RS_TOKENS_PATH` 優先を維持する（確認: `cargo test token_store_default_path` で期待パスを検証）
- [x] 旧パス `~/.config/slack-rs/tokens.json` から新パスへの自動移行処理を追加する（確認: 旧パスのみ存在するテストで初期化後に新パスへ同内容が作成される）
- [x] 移行時を含めて Unix の 0600 が維持されるようにする（確認: 既存のパーミッションテストと移行テストの双方で 0600 を検証）
- [x] 既存キー形式 `{team_id}:{user_id}` と `oauth-client-secret:{profile_name}` の互換性を回帰テストで固定する（確認: 既存キー互換テストが成功する）
- [x] 仕様差分を `file-token-storage` と `profiles-and-token-store` に反映する（確認: `npx @fission-ai/openspec@latest validate separate-credential-storage-path --strict` が成功する）

## Acceptance #1 Failure Follow-up

- [x] `tests/slack_token_env_tests.rs` の `test_fallback_to_token_store_when_slack_token_not_set` をテスト隔離する（`SLACK_RS_TOKENS_PATH` と設定パスを `tempdir` に固定し、実ホームの `~/.local/share/slack-rs/tokens.json` 依存を排除）
