- [x] `auth import` の内部結果モデルに profile 単位の action と理由を保持できるようにする（確認: 競合あり/なしの両ケースで結果件数が集計されるテストが成功）
- [x] `--yes` / `--force` 実行時でも更新・スキップ・上書きの結果サマリをテキスト出力する（確認: 統合テストで件数と対象 profile が出力される）
- [x] `auth import --json` で機械可読の結果配列を返す（確認: JSON パース可能で各 profile の `action` が検証できる）
- [x] `auth-export-import` の仕様差分を更新して結果報告要件を明文化する（確認: `npx @fission-ai/openspec@latest validate add-auth-import-machine-readable-reporting --strict` が成功）

- [x] `auth import --in <file> --yes` で team_id 競合がある場合に `Err(ProfileExists)` で処理全体を失敗させず、対象 profile を `action: skipped` として結果 (`profiles`/`summary`) に含めるよう `import_profiles` を修正する（確認: cargo test および cargo clippy 成功）
- [x] `auth import --in <file> --yes --force` で「別名 profile と同じ team_id」の競合が発生した場合、`action: updated` ではなく `action: overwritten` となるようにし、競合元 profile が上書きされる実装に修正する（確認: cargo test および cargo clippy 成功）

## Acceptance #2 Failure Follow-up

- [x] `auth import --in <file> --yes --force` の「別名 profile と同じ team_id」競合で、`action: overwritten` だけでなく実データ上も競合元 profile を置換するよう修正する（現状は `src/auth/export_import.rs` の `import_profiles` で競合名検出後も `config.set(name.clone(), profile)` のみ実行され、競合元 profile が残存する）（確認: cargo test および cargo clippy 成功）
