# タスク

- [x] 1. CLI コマンド設計（auth export/import の引数追加）
  - `--profile/--all`、`--out/--in`、`--passphrase-env`、`--passphrase-prompt`、`--yes`、`--force` を定義
  - 検証: `cargo test` の CLI 引数パーステストで各フラグが認識されること

- [x] 2. Export/Import データ構造と JSON 形式の実装
  - `format_version` と `profiles` を持つ payload を定義
  - unknown field を無視できる `serde` 設定にする
  - 検証: JSON round-trip テストで unknown field が無視されること

- [x] 3. 暗号化レイヤー（Argon2id + AES-256-GCM）実装
  - KDF params を保持し、nonce + ciphertext の format を確定
  - 検証: 固定 salt/nonce を使った round-trip テストが通ること

- [x] 4. ファイル権限チェックと安全な書き込み
  - export 作成時に 0600 を設定し、既存ファイルが 0600 以外ならエラー
  - 検証: 権限チェックのユニットテスト（unix のみ条件付き）

- [x] 5. Keyring ストレージの抽象化と mock 実装
  - 実 Keyring と in-memory mock を切り替えられる設計
  - 検証: mock を使った export/import の統合テストが通ること

- [x] 6. export コマンド実装
  - `--yes` が無い場合は中止
  - `--all`/`--profile` の挙動を分岐
  - 検証: CLI 統合テストで安全装置が有効なこと

- [x] 7. import コマンド実装
  - `team_id` 競合時はデフォルトで失敗し、`--yes` + `--force` で上書き
  - 検証: mock ストレージの競合ケーステストが通ること

- [x] 8. i18n メッセージの追加
  - ja/en の警告・プロンプト・エラーメッセージを追加
  - 検証: `--lang` 切り替えで該当文言が変化するテストが通ること

- [x] 9. 全体結合テスト
  - export → import の round-trip で同一 profile が復元される
  - 検証: `cargo test` が成功すること

## Acceptance #1 Failure Follow-up

- [x] `src/main.rs` の `handle_export_command` / `handle_import_command` で `--passphrase-env` が未設定のときに `--passphrase-prompt` へフォールバックする挙動を実装する
- [x] `src/auth/export_import.rs` の `import_profiles` で `--force` の上書きには `--yes` を必須にし、両方指定時のみ競合を許可する
- [x] `src/auth/export_import.rs` の `import_profiles` で team_id 競合検知を user_id 依存にせず、同一 team_id なら競合として扱う
- [x] `src/main.rs` の `handle_export_command` で `--yes` 未指定時にも危険操作の警告を表示して中止する

## Acceptance #2 Failure Follow-up

- [x] `src/main.rs` の `handle_export_command` / `handle_import_command` で `--passphrase-env` を指定して環境変数が未設定の場合に `--passphrase-prompt` にフォールバックして対話入力できるようにする
