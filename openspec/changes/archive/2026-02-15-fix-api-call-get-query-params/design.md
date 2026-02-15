# Design: GET query params for api call

## 方針
- `ApiClient::call` に query params を渡せるようにする
- GET の場合のみ query を付与し、body は送らない
- POST の場合は従来通り JSON / フォーム送信を継続する

## 変更点
### 1) API クライアント
- `ApiClient::call` の引数に query params を追加する
- `execute_request` で `request.query(&query_params)` を適用する
  - これにより GET のみならず将来の拡張にも対応可能

### 2) api call 実行
- `execute_api_call` で GET 時に `args.to_form()` を query として用いる
- POST 時は query を空にし、body は `--json` / フォームの既存ロジックを維持する

## 互換性
- `ApiClient::call` の呼び出しは `execute_api_call` のみであり、変更は局所的
- `--json` と `--get` の同時指定は、GET の query 優先として解釈する（body 送信なし）

## テスト方針
- 既存の `test_api_call_with_get_method` に query 条件を追加
- `conversations.replies` の GET + query を確認するテストを追加
- 外部依存は httpmock で完結させる
