- [ ] 1. `auth login` の引数に `--ngrok [path]` を追加し、`--cloudflared` との同時指定をエラーにする。
  - 検証: 引数パースのテストで `--ngrok` の有無と競合エラーを確認する。

- [ ] 2. ngrok 実行ファイルの解決を実装する。
  - `--ngrok` のみ指定された場合は `ngrok` を PATH から探索して使用する。
  - `--ngrok <path>` が指定された場合はそのパスを使用する。
  - 検証: 入出力モックで解決結果が正しいことを確認する。

- [ ] 3. `auth login` に ngrok tunnel 起動・停止を統合する。
  - `ngrok http 8765` を起動し、出力から `https://*.ngrok-free.app` を抽出する。
  - redirect_uri を `{public_url}/callback` として OAuth フローに渡す。
  - OAuth 完了後に ngrok プロセスを停止する。
  - 検証（モックファースト）:
    - ngrok 出力をモックして URL 抽出が行われることを確認する。
    - base_url 上書きで OAuth エンドポイントをモックし、外部接続なしで完了することを確認する。

- [ ] 4. Manifest 生成に ngrok 分岐を追加する。
  - ngrok 使用時は `oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` を含める。
  - ngrok 非使用時は既存の redirect_uri ルールを維持する。
  - 検証: 固定入力に対して期待する YAML が生成されるユニットテストを追加する。

- [ ] 5. CLI ドキュメント/ヘルプに `--ngrok [path]` を追加し、注意事項（カスタムドメインは対象外）を明記する。
  - 検証: `--help` 出力に `--ngrok` の説明が含まれることを確認する。
