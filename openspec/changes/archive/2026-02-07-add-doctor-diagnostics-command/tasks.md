- [x] `doctor` コマンドのエントリポイントを追加し、既存コマンドルーティングへ接続する（確認: `slack-rs doctor --help` が表示される）
- [x] 診断出力に `configPath`、token store backend/path、bot/user token 有無を含める（確認: モック構成で期待キーが出力される統合テストが成功）
- [x] トークン値を出力しない安全制約を追加する（確認: 出力に `xoxb-` / `xoxp-` が含まれないことをテストで検証）
- [x] `doctor --json` の機械可読出力を追加する（確認: JSON パース可能で必須フィールドを検証できる）
- [x] 新規 capability の仕様差分を追加し、コマンド追加を明文化する（確認: `npx @fission-ai/openspec@latest validate add-doctor-diagnostics-command --strict` が成功）

## Acceptance #1 Failure Follow-up

- [x] `doctor --json` の JSON キーを仕様どおり camelCase に修正する（`configPath` / `tokenStore` / `botTokenExists` / `userTokenExists`）。`src/commands/doctor.rs` の `DiagnosticInfo` / `TokenStoreInfo` / `TokenStatus` に `serde(rename_all = "camelCase")` を適用し、`tests/doctor_integration_tests.rs` のスキーマ検証も同じキーに更新する。
- [x] `doctor --help` で診断実行ではなくヘルプ表示を返すように CLI ルーティングを修正する。`src/main.rs` の `"doctor"` 分岐で `--help` / `-h` を先に処理し、期待出力を `tests/doctor_integration_tests.rs`（またはコマンド統合テスト）に追加する。
