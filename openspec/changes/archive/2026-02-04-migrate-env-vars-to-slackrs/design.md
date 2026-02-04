## 方針

- 環境変数の接頭辞は `SLACKRS_` に統一する。
- `SLACKCLI_*` は互換用途としても参照しない（移行期間は設けない）。
- エラーメッセージ、ヘルプ、ドキュメントの表記はすべて新しい名称へ更新する。

## 対象となる環境変数

- `SLACKRS_CLIENT_ID`
- `SLACKRS_CLIENT_SECRET`
- `SLACKRS_REDIRECT_URI`
- `SLACKRS_SCOPES`
- ドキュメント中の `SLACKCLI_KEYRING_PASSWORD` / `SLACKCLI_AUDIT` / `SLACKCLI_ENABLE_WRITE` / `SLACKCLI_LANG` などの表記も `SLACKRS_*` に合わせる

## テスト方針

- `src/main.rs` の OAuth 設定読み込みが `SLACKRS_*` を参照していることをレビューで確認する。
- CLI 実行時のエラーメッセージが新しい変数名になっていることを確認する。
