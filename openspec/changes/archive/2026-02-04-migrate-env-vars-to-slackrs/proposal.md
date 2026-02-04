## Why

環境変数が `SLACKCLI_*` のままになっており、バイナリ名 `slack-rs` と整合していません。利用者の設定ミスやドキュメントの混乱を避けるため、環境変数の接頭辞を `SLACKRS_*` に統一します。

## What Changes

- OAuth 設定に使う環境変数を `SLACKRS_*` に変更する
- エラーメッセージや使用例の表示を新しい環境変数名に合わせる
- README / docs の環境変数表記を `SLACKRS_*` に更新する

## Capabilities

### New Capabilities
- (なし)

### Modified Capabilities
- `oauth-login`: 必須の OAuth 環境変数名を `SLACKRS_*` に更新する

## Impact

- `src/main.rs`（OAuth 設定の読み込みとエラーメッセージ）
- `README.md` / `docs/oauth.md` / `docs/basic-design.md` / `docs/security.md` / `docs/SlackSearchCLI_Rust_Spec.md` / `docs/SlackCLI_AuthExportImport_Spec.md`
