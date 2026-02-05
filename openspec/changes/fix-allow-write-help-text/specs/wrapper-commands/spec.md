## MODIFIED Requirements

### Requirement: Write 操作のヘルプは環境変数を示す
write 操作の usage/ヘルプは `SLACKCLI_ALLOW_WRITE` による制御を明示し、`--allow-write` が必須であると誤認させてはならない。(MUST)

#### Scenario: ヘルプ表示で write 制御が明示される
- Given `slack-rs` のヘルプ/usage を表示する
- When `msg`/`react` など write 操作の説明を確認する
- Then `SLACKCLI_ALLOW_WRITE` による制御が示される
- And `requires --allow-write` のような必須表記が含まれない
