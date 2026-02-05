## ADDED Requirements
### Requirement: Wrapper commands show guidance for known Slack error codes
ラッパーコマンドの実行結果が `ok=false` かつ `error` が既知のコードに一致する場合、標準エラー出力に原因と解決策のガイダンスを表示すること。JSON 出力の内容は変更せず、追加情報は標準エラー出力に限定すること。(MUST)

#### Scenario: `missing_scope` のガイダンスが表示される
- Given Slack API が `ok=false` と `error=missing_scope` を返す
- When `conv list --types=private_channel` を実行する
- Then 標準エラー出力に原因と解決策が表示される
- And JSON 出力は Slack のレスポンスのままである
