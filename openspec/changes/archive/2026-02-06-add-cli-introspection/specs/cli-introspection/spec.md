# cli-introspection Specification

## Purpose
CLI が自身のコマンド構造と出力仕様を機械可読に提示できるようにし、エージェントが安全に発見・理解・検証できるようにする。

## ADDED Requirements

### Requirement: `commands --json` はコマンド一覧を JSON で返す
CLI は `commands --json` を提供し、サブコマンドとフラグ一覧を機械可読な JSON で返さなければならない。(MUST)

#### Scenario: `slack-rs commands --json` を実行した場合
- 出力は JSON としてパース可能である
- `schemaVersion`/`type`/`ok` が含まれる
- `commands` 配列に `msg post` と `conv list` が含まれる

### Requirement: `--help --json` は構造化ヘルプを返す
任意のコマンドに対して `--help --json` を指定した場合、構造化ヘルプを JSON で返さなければならない。(MUST)

#### Scenario: `slack-rs msg post --help --json` を実行した場合
- `command` と `usage` が含まれる
- `flags` に `--thread-ts` が含まれる
- `examples` と `exitCodes` が含まれる

### Requirement: `schema --command <cmd> --output json-schema` は出力スキーマを返す
CLI は `schema --command <cmd> --output json-schema` を提供し、コマンド出力の JSON Schema を返さなければならない。(MUST)

#### Scenario: `slack-rs schema --command conv.list --output json-schema` を実行した場合
- JSON Schema としてパース可能である
- `schemaVersion`/`type`/`ok` のフィールド定義を含む
- `response` と `meta` のフィールド定義を含む

### Requirement: 統一エンベロープは `schemaVersion`/`type`/`ok` を含む
`--raw` 以外の JSON 出力は `schemaVersion`/`type`/`ok` を含む統一エンベロープを返さなければならない。(MUST)

#### Scenario: `slack-rs conv list` を実行した場合
- 出力に `schemaVersion` が含まれる
- 出力に `type` が含まれる
- 出力に `ok` が含まれる
