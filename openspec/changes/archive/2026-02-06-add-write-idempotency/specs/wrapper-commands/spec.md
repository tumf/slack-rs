# wrapper-commands Specification

## MODIFIED Requirements
### Requirement: Destructive operations require confirmation without --yes flag
write 操作（`msg post/update/delete`, `react add/remove`, `file upload`）はデフォルトで確認を求めなければならない。(MUST)
`--yes` が指定されている場合は確認を省略しなければならない。(MUST)
非対話モードでは `--yes` が無い場合に即時エラーにしなければならない。(MUST)
write 操作のヘルプ/使用例には `--idempotency-key` の説明を含めなければならない。(MUST)

#### Scenario: ヘルプに `--idempotency-key` が表示される
- Given write 操作のヘルプ表示を確認する
- When `msg post --help` を表示する
- Then `--idempotency-key` の説明が含まれる
