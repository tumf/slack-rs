1. OAuth 設定の環境変数名を `SLACKRS_*` に置き換える
   - 検証: `src/main.rs` で `SLACKRS_CLIENT_ID` / `SLACKRS_CLIENT_SECRET` / `SLACKRS_REDIRECT_URI` / `SLACKRS_SCOPES` を参照していることを確認する
2. OAuth 必須値未設定時のエラーメッセージと使用例を更新する
   - 検証: `src/main.rs` のエラーメッセージに `SLACKRS_CLIENT_ID` / `SLACKRS_CLIENT_SECRET` が含まれていることを確認する
3. ドキュメントの環境変数表記を `SLACKRS_*` に統一する
   - 検証: `README.md`, `docs/oauth.md`, `docs/basic-design.md`, `docs/security.md`, `docs/SlackSearchCLI_Rust_Spec.md`, `docs/SlackCLI_AuthExportImport_Spec.md` の該当箇所が更新されていることを確認する
