# cli-help-recipes Specification

## Purpose
利用頻度の高い操作を即座に試せるよう、help と recipes を整備する。

## ADDED Requirements

### Requirement: Help includes copy/paste examples for key tasks
`slack-rs --help` には profile 選択と出力切替の例を含めなければならない。(MUST)

#### Scenario: `slack-rs --help` の例が表示される
- When `slack-rs --help` を実行する
- Then profile 選択と出力切替の例が含まれる

### Requirement: `docs/recipes.md` provides common workflows
`docs/recipes.md` を追加し、以下の項目を含めなければならない。(MUST)
- profile の選択方法
- raw/envelope 出力の切替
- 会話/スレッドの読み取り
- 代表的なエラーの意味と対処

#### Scenario: recipes の見出しが存在する
- When `docs/recipes.md` を開く
- Then 上記の項目に対応する見出しが存在する
