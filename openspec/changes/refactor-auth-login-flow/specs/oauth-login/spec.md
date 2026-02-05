# oauth-login 変更提案

## MODIFIED Requirements
### Requirement: auth login の仕様を維持したまま内部構造を整理する
`auth login` は既存の引数、対話入力、デフォルト値、エラーハンドリングを変更せずに動作しなければならない。(MUST)

#### Scenario:
- Given `auth login` に既存のフラグと入力を与える
- When OAuth フローを開始する
- Then 既存と同じ入力プロンプトとエラー条件が適用される
