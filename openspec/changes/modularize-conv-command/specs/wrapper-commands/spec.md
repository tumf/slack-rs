# wrapper-commands 変更提案

## MODIFIED Requirements
### Requirement: conv コマンドの内部モジュール分割後も挙動は維持される
`conv list`/`conv select`/`conv history` は内部構成を分割しても、既存の引数・出力・エラー挙動を維持しなければならない。(MUST)

#### Scenario:
- Given 既存の `conv` コマンド引数を使用する
- When `conv list` または `conv history` を実行する
- Then 以前と同じ API 呼び出しと出力形式が維持される
