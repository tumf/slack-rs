# 設計: 出力エンベロープの統一

## 目的
JSON 出力を統一し、スクリプト利用時の一貫性を確保する。必要に応じて raw 出力へ戻れるようにする。

## 出力フォーマット
デフォルトは以下のエンベロープを返す。

```json
{
  "response": { "ok": true, "channels": [] },
  "meta": {
    "profile_name": "default",
    "team_id": "T0EXAMPLE",
    "user_id": "U0EXAMPLE",
    "method": "conversations.list",
    "command": "conv list"
  }
}
```

### `meta` に含める情報
- `profile_name`: 省略時は `null`
- `team_id`, `user_id`: 実行時の解決結果
- `method`: Slack API メソッド名
- `command`: CLI コマンド名（例: `api call`, `conv list`）

## `--raw` オプション
`--raw` 指定時は Slack API レスポンスをそのまま返し、`response`/`meta` のラップを行わない。

## 互換性方針
- 既存のスクリプト利用者向けに `--raw` を用意して段階移行を支援する
- README と `--help` に出力形式と移行方法を明記する
