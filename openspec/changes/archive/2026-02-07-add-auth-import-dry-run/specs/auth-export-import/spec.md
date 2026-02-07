## ADDED Requirements

### Requirement: `auth import --dry-run` は書き込みなしで適用計画を表示する
`auth import --dry-run` は import 判定を実行するが、設定ファイルおよび token store への書き込みを行ってはならない。(MUST NOT)

dry-run 実行時は profile 単位の予定 action を出力しなければならない。(MUST)

#### Scenario: dry-run では書き込みせず予定のみ表示する
- Given 既存 profile と衝突する import データがある
- When `auth import --dry-run` を実行する
- Then `profiles.json` と token store の内容は変更されない
- And 各 profile の予定 action が表示される

### Requirement: dry-run は予定 action (created/updated/skipped/overwritten) を明示する
dry-run 実行時の出力は、各 profile に対する予定 action を含まなければならない。(MUST)

action は以下のいずれかでなければならない。(MUST)
- `created`: 新規 profile として作成予定
- `updated`: 既存 profile (同一 team_id) を更新予定
- `skipped`: 衝突のためスキップ予定 (--force なし)
- `overwritten`: 衝突だが上書き予定 (--force あり)

#### Scenario: 各 profile の action が表示される
- Given 新規 profile、更新 profile、衝突 profile を含む import データ
- When `auth import --dry-run` を実行する
- Then 各 profile に対する action (created/updated/skipped/overwritten) が表示される
- And action の理由 (reason) も表示される

### Requirement: `--dry-run --json` は機械可読の予定結果を返す
`--json` フラグと `--dry-run` の併用時、人間可読形式ではなく JSON 形式で予定結果を出力しなければならない。(MUST)

JSON 出力は以下の構造を持たなければならない。(MUST)
- `dry_run`: boolean (true/false)
- `profiles`: array of objects
  - `profile_name`: string
  - `action`: "created" | "updated" | "skipped" | "overwritten"
  - `team_id`: string
  - `user_id`: string
  - `reason`: string | null

#### Scenario: JSON 形式で予定結果を返す
- Given import データ
- When `auth import --dry-run --json` を実行する
- Then JSON 形式で予定結果が出力される
- And 各 profile の action, team_id, user_id, reason が含まれる

### Requirement: `--force` との併用時は上書き予定を報告する
`--dry-run` と `--force` を併用した場合、衝突する profile は `overwritten` として報告されなければならない。(MUST)

#### Scenario: force 時は上書き予定として報告
- Given 既存 profile と team_id が異なる import データ
- When `auth import --dry-run --force` を実行する
- Then 衝突する profile の action は `overwritten` となる
- And 実際の書き込みは行われない
