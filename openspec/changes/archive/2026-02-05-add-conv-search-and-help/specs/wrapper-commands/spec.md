## ADDED Requirements
### Requirement: `conv search` は名前で検索できる
`conv search <pattern>` は `conversations.list` の結果に対して `name:<pattern>` のフィルタを適用し、結果を出力しなければならない。(MUST)
`conv search` は `--types`/`--limit`/`--format`/`--sort`/`--sort-dir` を `conv list` と同様に受け付けなければならない。(MUST)

#### Scenario: `conv search` で名前パターン検索する
- Given `conv search "ark*" --format jsonl` を実行する
- When 会話一覧を取得する
- Then `name:ark*` フィルタが適用される
- And 出力は `jsonl` 形式である

### Requirement: `conv search --select` はチャンネル ID を返す
`conv search --select` はフィルタ後の一覧から対話的に 1 件を選択し、チャンネル ID を出力しなければならない。(MUST)

#### Scenario: `--select` でチャンネル ID を返す
- Given `conv search "project" --select` を実行する
- When 対話でチャンネルを選択する
- Then 選択したチャンネル ID が標準出力に出力される

### Requirement: `conv list` のヘルプはフィルタ/フォーマット/ソートを説明する
`conv list --help` は `--filter` のキー一覧（`name`/`is_member`/`is_private`）と、`--format` の有効値（`json`/`jsonl`/`table`/`tsv`）、`--sort`/`--sort-dir` の有効値を明記しなければならない。(MUST)

#### Scenario: `conv list --help` で説明が表示される
- Given `conv list --help` を実行する
- When ヘルプ出力を確認する
- Then フィルタ/フォーマット/ソートの有効値が含まれる
