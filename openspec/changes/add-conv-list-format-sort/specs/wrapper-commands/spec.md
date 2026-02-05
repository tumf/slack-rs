## ADDED Requirements

### Requirement: `conv list` は `--format` で出力形式を切り替えられる
`conv list` は `--format` で `json`/`jsonl`/`table`/`tsv` を受け付け、指定された形式で結果を出力しなければならない。(MUST)
`--format` が未指定の場合は、従来と同等の JSON 出力を維持しなければならない。(MUST)

#### Scenario: `--format jsonl` は 1 行 1 チャンネルで出力される
- Given 有効な profile と token が存在する
- When `conv list --format jsonl` を実行する
- Then 出力はチャンネルごとに 1 行の JSON である

#### Scenario: 不正な `--format` はエラーになる
- Given `conv list --format nope` を実行する
- When 出力形式の解釈に失敗する
- Then 許容値（`json`/`jsonl`/`table`/`tsv`）を含むエラーが表示される

### Requirement: `conv list` は `table`/`tsv` で `num_members` の欠落値を空欄として出力する
Slack API の応答で `channel.num_members` が欠落または `null` の場合、`--format table` および `--format tsv` は `num_members` を空欄として出力しなければならない。(MUST)
このとき `num_members` を `0` として出力してはならない（不明の意味を保持するため）。(MUST)

#### Scenario: `--format tsv` で `num_members` が欠落している場合は空欄で出力される
- Given Slack API の取得結果に `num_members` が欠落（または `null`）のチャンネルが含まれる
- When `conv list --format tsv` を実行する
- Then 該当チャンネルの出力行において `num_members` フィールドは空欄である（例: 最終フィールドが空のタブ区切りになる）

#### Scenario: `--format table` で `num_members` が欠落している場合は空欄で出力される
- Given Slack API の取得結果に `num_members` が欠落（または `null`）のチャンネルが含まれる
- When `conv list --format table` を実行する
- Then 該当チャンネルの `num_members` 列は空欄として表示される

### Requirement: `conv list` の `--raw` は JSON 出力時のみ有効である
`--raw` は出力フォーマットが JSON のとき（デフォルト/`--format json`）のみ有効/意味がある。(MUST)
`--format` が `jsonl`/`table`/`tsv` の場合に `--raw` が指定されたとき、コマンドはエラーにしなければならない。(MUST)
エラーメッセージは `--raw` と指定された `--format` が互換でないことを示さなければならない。(MUST)

#### Scenario: `--format tsv` と `--raw` の併用はエラーになる
- Given `conv list --format tsv --raw` を実行する
- When オプションの整合性検証に失敗する
- Then `--raw` と `--format tsv` が互換でないことを示すエラーが表示される

### Requirement: `conv list` は `--sort` と `--sort-dir` で結果を並び替えられる
`conv list` は `--sort` が指定された場合、取得結果に対して `name`/`created`/`num_members` のいずれかでソートを適用しなければならない。(MUST)
`--sort-dir` は `asc`/`desc` を受け付け、ソート方向を制御しなければならない。(MUST)
ソートは `--filter` による絞り込みの後に適用されなければならない。(MUST)

#### Scenario: `name` で降順ソートする
- Given 取得結果に `name` が異なる複数のチャンネルが含まれる
- When `conv list --sort name --sort-dir desc --format tsv` を実行する
- Then 出力行は `name` の降順に並ぶ

#### Scenario: 不正な `--sort`/`--sort-dir` はエラーになる
- Given `conv list --sort nope --sort-dir sideways` を実行する
- When ソート指定の解釈に失敗する
- Then 許容値（`name`/`created`/`num_members`, `asc`/`desc`）を含むエラーが表示される
