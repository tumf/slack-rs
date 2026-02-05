## MODIFIED Requirements
### Requirement: Wrapper commands show guidance for known Slack error codes
ラッパーコマンドの実行結果が `ok=false` かつ `error` が既知のコードに一致する場合、標準エラー出力に `Error:`/`Cause:`/`Resolution:` を含むガイダンスを表示しなければならない。(MUST)

#### Scenario: `missing_scope` のガイダンスが表示される
- Given Slack API が `ok=false` と `error=missing_scope` を返す
- When `users info --user U123` を実行する
- Then 標準エラー出力に `Error:`/`Cause:`/`Resolution:` が含まれる
- And JSON 出力は Slack のレスポンスのままである
