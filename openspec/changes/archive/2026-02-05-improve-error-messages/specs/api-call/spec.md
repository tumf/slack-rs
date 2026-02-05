## ADDED Requirements
### Requirement: Provide guidance for known Slack error codes
`api call` の実行結果が `ok=false` かつ `error` が既知のコード（`not_allowed_token_type`, `missing_scope`, `invalid_auth` など）に一致する場合、標準エラー出力に原因と解決策のガイダンスを表示すること。JSON 出力の `response` は変更せず、追加情報は標準エラー出力に限定すること。(MUST)

#### Scenario: `not_allowed_token_type` のガイダンスが表示される
- Given Slack API が `ok=false` と `error=not_allowed_token_type` を返す
- When `api call search.messages` を実行する
- Then 標準エラー出力に原因と解決策が表示される
- And JSON 出力の `response` は Slack のレスポンスのままである
