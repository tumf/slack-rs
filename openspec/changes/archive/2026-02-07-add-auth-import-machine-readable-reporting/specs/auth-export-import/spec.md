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

#### Scenario: テキスト出力でサマリと詳細が表示される
- Given import 可能な profile が含まれるファイルがある
- When `auth import --in <file> --yes` を実行する
- Then `Import Summary:` セクションに `Total`, `Updated`, `Skipped`, `Overwritten` 件数が表示される
- And `Profile Details:` セクションに各 profile の名前、action、理由が表示される

#### Scenario: JSON 出力で機械可読な結果が返される
- Given import 可能な profile が含まれるファイルがある
- When `auth import --in <file> --yes --json` を実行する
- Then 出力は有効な JSON フォーマットである
- And `profiles` 配列に各 profile の `profile_name`, `action`, `reason` が含まれる
- And `summary` オブジェクトに `total`, `updated`, `skipped`, `overwritten` が含まれる
- And `action` は `"updated"`, `"skipped"`, `"overwritten"` のいずれかである

#### Scenario: profile が新規追加される場合 action は updated
- Given 存在しない profile 名の import データがある
- When `auth import --in <file> --yes` を実行する
- Then その profile の action は `updated` である
- And reason には "New profile imported" が含まれる

#### Scenario: 同じ team_id の profile を更新する場合 action は updated
- Given 既存 profile と同じ team_id の import データがある
- When `auth import --in <file> --yes` を実行する (--force なし)
- Then その profile の action は `updated` である
- And reason には "Updated existing profile" と team_id が含まれる

#### Scenario: team_id が衝突する profile は --force なしで skipped
- Given 異なる profile 名だが同じ team_id の import データがある
- When `auth import --in <file> --yes` を実行する (--force なし)
- Then その profile の action は `skipped` である
- And reason には team_id conflict の情報が含まれる

#### Scenario: --force 指定時は衝突する profile が overwritten
- Given 既存 profile と衝突する import データがある
- When `auth import --in <file> --yes --force` を実行する
- Then その profile の action は `overwritten` である
- And reason には "Overwritten" の情報が含まれる
