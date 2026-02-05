- [x] token store backend の解決ロジックを追加する（確認: `SLACKRS_TOKEN_STORE` 未設定時は Keyring が選択され、`file` 指定時は FileTokenStore が選択される）
- [x] Keyring 利用不能時にデフォルト設定でコマンドが失敗し、実行可能なガイダンスを表示する（確認: Keyring を stub で失敗させ、エラーに `SLACKRS_TOKEN_STORE=file` の案内が含まれる）
- [x] Keyring がロックされていて interaction-required 相当のエラーになる場合に、繰り返しプロンプトせず失敗し、OS の Keyring アンロックまたは `SLACKRS_TOKEN_STORE=file` のガイダンスを表示するテストを追加する
- [x] file mode で既存の `tokens.json` パスとキー形式を再利用する（確認: 既存の `~/.config/slack-rs/tokens.json` を InMemoryTokenStore で置き換えたテストでキーが一致する）
- [x] `config oauth show` が backend に関わらず `client_secret` を出力しない（確認: `client_secret` を保存した状態で `show` を実行し、出力に `client_secret` の値が含まれない）
- [x] 仕様に対応するユニットテストを mock-first で追加する（確認: InMemoryTokenStore を用いた保存/取得、Keyring 失敗 stub、`SLACKRS_TOKEN_STORE` の分岐がテストされる）
- [x] 既存の関連テストを実行し回帰がないことを確認する（確認: `cargo test` が成功する）

## Acceptance #1 Failure Follow-up
- [x] `config`/`auth`/`cli`/`main` の各フローで `FileTokenStore::new()` 直呼びをやめ、`create_token_store` を使って backend を解決する（未指定時は Keyring、`SLACKRS_TOKEN_STORE=file` のみ file）。
- [x] Keyring 利用不能時に `TokenStoreError::KeyringUnavailable` を伝播して失敗し、ガイダンスを表示する（無言の file フォールバック禁止）。
