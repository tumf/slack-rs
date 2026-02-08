# 実装タスク

- [x] `src/skills/mod.rs`（新規）に `install-skill` のコア実装を追加し、`Source` 解析を `self` と `local:<path>` のみに制限する（未知スキームは即時エラー）。
  検証: `cargo test skills::tests::parse_source_accepts_self_and_local --lib` と `cargo test skills::tests::parse_source_rejects_unknown_scheme --lib` が成功する。

- [x] `skills/slack-rs` を `self` ソースとして読み出せる埋め込みローダーを追加し、引数なし時の既定ソースを必ず `self` に固定する（外部 URL/GitHub 解決を実装しない）。
  検証: `cargo test skills::tests::default_source_is_self --lib` と `cargo test skills::tests::self_source_uses_embedded_skill --lib` が成功する。

- [x] インストール処理を実装し、配置先を `~/.config/slack-rs/.agents/skills/<skill-name>`、ロックを `~/.config/slack-rs/.agents/.skill-lock.json` に更新する。
  検証: `cargo test skills::tests::install_writes_skill_dir_and_lock_file --lib` が成功し、テスト内で上記2パス相当の出力が確認できる。

- [x] 配置戦略を「symlink 優先、失敗時 copy フォールバック」に実装する。
  検証: `cargo test skills::tests::falls_back_to_copy_when_symlink_fails --lib` が成功する。

- [x] `install-skill` 成功時の JSON 出力（`schemaVersion`/`type`/`ok`/`skills[].name`/`skills[].path`/`skills[].source_type`）と失敗時の stderr + 非ゼロ終了を実装する。
  検証: `cargo test --test skill_installation_integration install_skill_outputs_required_json_fields` と `cargo test --test skill_installation_integration install_skill_invalid_source_exits_non_zero` が成功する。

- [x] 実装コードへの配線（entry-point）として `src/main.rs` に `install-skill` ルーティングを追加し、`print_help`/`print_usage` にコマンド説明を追加する。
  検証: `cargo test --test skill_installation_integration install_skill_is_routed_from_main` が成功する。

- [x] イントロスペクション配線として `src/cli/introspection.rs` の `commands --json` 一覧に `install-skill` を追加し、`schema --command install-skill --output json-schema` でスキーマが返るようにする。
  検証: `cargo test --test skill_installation_integration commands_json_includes_install_skill` と `cargo test --test skill_installation_integration schema_for_install_skill_is_available` が成功する。

- [x] すべての検証を外部依存なしで再実行できるよう、テストは一時ディレクトリとリポジトリ内 fixture（`skills/slack-rs`）のみを使用し、ネットワークアクセス不要に固定する。
  検証: `cargo test --lib skills::tests` と `cargo test --test skill_installation_integration` がオフライン前提で成功する。

## Acceptance #1 Failure Follow-up

- [x] `slack-rs install-skill foo:bar` の失敗時エラーメッセージに、許容スキーム一覧（`self`, `local:<path>`）を含めるよう修正し、`src/skills/mod.rs` と統合テストで検証する。
- [x] `cargo test --lib` で失敗している `cli::handlers::tests::test_resolve_token_fallback_from_user_to_bot`（`src/cli/handlers.rs`）を解消し、`resolve_token` の実装またはテスト期待値を現仕様と一致させる。
