## Implementation Tasks

- [x] 1. `--cloudflared` / `--ngrok` 経路の入力解決順序を manifest-first に再構成する（verification: integration - `src/cli/handlers.rs` と `src/auth/commands.rs` で credentials 解決が manifest 案内後へ移動していることを確認し、`cargo test --lib`）
- [x] 2. manifest 生成 API から不要な `client_id` 依存を外し、redirect URI + scopes + profile 情報だけで生成できることを明示する（verification: integration - `src/auth/manifest.rs` と `tests/manifest_generation_integration.rs`, `cargo test --test manifest_generation_integration`）
- [x] 3. app 作成後に credentials を解決する UX を定義し、保存済み設定の再利用または対話入力のどちらでも manifest-first を壊さないようにする（verification: unit - `src/auth/commands.rs` の `login_with_credentials_extended` が `resolve_client_secret` と既存プロファイルからの `client_id` 再利用を実装）
- [x] 4. README とヘルプ文言を manifest-first 手順へ更新する（verification: manual - `README.md` と `src/main.rs` の案内が「manifest -> app 作成 -> credentials -> OAuth」に一致することを確認）
- [x] 5. 変更後のログイン関連回帰を実行する（verification: integration - `cargo test --lib` (424 passed), `cargo test --test manifest_generation_integration` (4 passed); lint - `cargo clippy -- -D warnings` (clean); format - `cargo fmt -- --check` (clean)）

## Future Work

- 実際の Slack App 作成画面での手動確認。
- 必要に応じた `config oauth set` との UX 統合整理。
