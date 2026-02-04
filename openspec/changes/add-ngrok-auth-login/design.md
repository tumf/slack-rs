## 設計方針

### 追加オプション
- `auth login` に `--ngrok [path]` を追加する。
- `path` が省略された場合は実行ファイル名 `ngrok`（PATH 探索）を使用する。
- `--ngrok` と `--cloudflared` の同時指定はエラーとする。

### ngrok による redirect_uri 解決
- `--ngrok [path]` が指定された場合、`ngrok http 8765` を起動する。
- ngrok の標準出力から公開 URL（例: `https://xxxx.ngrok-free.app`）を抽出する。
- redirect_uri は `{public_url}/callback` を使用する。
- OAuth フロー完了後に ngrok プロセスを停止する。

### Manifest 生成
- ngrok を使用した場合、`oauth_config.redirect_urls` に `https://*.ngrok-free.app/callback` を含める。
- ngrok を使用しない場合の挙動は既存仕様に従う。
- ngrok のカスタムドメインは本変更の対象外とする。

### エラーハンドリング
- `--ngrok [path]` が指定されているのに実行できない場合は、明確なエラーメッセージを表示して OAuth を開始しない。
- `--ngrok` と `--cloudflared` が同時に指定された場合は即時エラーとする。

### テスト方針
- ngrok のプロセスはモックし、出力から URL 抽出が行われることを検証する。
- OAuth エンドポイントは既存の base_url 上書きを使ってモックする。
- Slack UI への実アップロードはテスト対象外とする。
