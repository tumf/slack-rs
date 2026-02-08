## MODIFIED Requirements

### Requirement: API 呼び出し時のトークン解決規約は内部責務分離後も維持される
`run_api_call` の内部でトークン解決ロジックを分離した後も、トークン解決優先順位（CLI 指定 > profile 既定 > 推論）と `SLACK_TOKEN` 優先、明示指定時の厳格エラー、必要時のフォールバック挙動は維持されなければならない。(MUST)

#### Scenario: 分離後も `SLACK_TOKEN` 優先が維持される
- Given profile に既定トークンタイプが設定され token store にもトークンがある
- And `SLACK_TOKEN` 環境変数が設定されている
- When `api call` を実行する
- Then 認証に使用されるトークンは `SLACK_TOKEN` である
- And 既存のメタ情報とレスポンス形式は維持される
