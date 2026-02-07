## ADDED Requirements

### Requirement: `auth import --dry-run` は書き込みなしで適用計画を表示する
`auth import --dry-run` は import 判定を実行するが、設定ファイルおよび token store への書き込みを行ってはならない。(MUST NOT)

dry-run 実行時は profile 単位の予定 action を出力しなければならない。(MUST)

#### Scenario: dry-run では書き込みせず予定のみ表示する
- Given 既存 profile と衝突する import データがある
- When `auth import --dry-run` を実行する
- Then `profiles.json` と token store の内容は変更されない
- And 各 profile の予定 action が表示される
