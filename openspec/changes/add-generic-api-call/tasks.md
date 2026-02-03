# タスク

- [x] HTTP クライアントを実装し、base URL を差し替え可能にする（検証: モックサーバでリクエスト受信を確認）
- [x] `api call` の引数解析（method, key=value, --json, --get）を実装する（検証: 引数解析ユニットテスト）
- [x] form と JSON の送信を実装する（検証: モックサーバで Content-Type と body を検証）
- [x] 429 / Retry-After を考慮したリトライを実装する（検証: 429 を返すモックサーバで待機後再試行を確認）
- [x] 出力 JSON に meta を付与する（検証: `meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method` を確認）
- [x] `api call` を CLI ルーティングに接続する（検証: `--help` に api call が表示されることを確認）
