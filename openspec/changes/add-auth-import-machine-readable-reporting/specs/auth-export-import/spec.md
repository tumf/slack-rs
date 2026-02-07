## MODIFIED Requirements

### Requirement: Import applies safeguard on team_id conflict
`auth import` は衝突時の処理結果を実行後に報告しなければならない。(MUST)

`--yes` や `--force` の有無にかかわらず、更新・スキップ・上書きの件数と対象を取得できなければならない。(MUST)

`--json` 指定時は profile 単位の結果を機械可読形式で返さなければならない。(MUST)

#### Scenario: `--force --json` で profile 単位の結果を取得できる
- Given 衝突する profile を含む import ファイルがある
- When `auth import --force --json` を実行する
- Then 出力 JSON には profile ごとの `action` が含まれる
- And `updated` / `skipped` / `overwritten` の集計が取得できる
