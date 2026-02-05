# auth-export-import 変更提案

## MODIFIED Requirements
### Requirement: export/import の引数解釈は共通化しても挙動を維持する
`auth export` と `auth import` は共通パーサーを使用しても、既存のフラグ・確認・エラーの挙動を維持しなければならない。(MUST)

#### Scenario:
- Given 既存の `auth export`/`auth import` の引数セットを使用する
- When 各コマンドを実行する
- Then 既存と同じ確認フローとエラー条件が適用される
