## ADDED Requirements

### Requirement: `tokens.json` のシリアライズは決定的である
`FileTokenStore` は `tokens.json` 保存時にキー順序を決定的にし、同一内容からは同一の論理出力を生成しなければならない。(MUST)

意味的に変更がない場合、再保存で不要な差分を発生させてはならない。(MUST NOT)

#### Scenario: 同一内容を保存した場合は安定した出力になる
- Given 同じキーと値を異なる挿入順で保存する
- When `tokens.json` を書き出す
- Then 出力されるキー順序は一貫している
- And 内容不変の再保存で不要な差分が発生しない
