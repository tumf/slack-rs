# wrapper-commands

## MODIFIED Requirements

### Requirement: `conv list` はスペース区切りのオプションを受け付ける
`conv list` の値付きオプション（`--filter`/`--format`/`--sort`/`--sort-dir`/`--types`/`--limit`/`--profile`）は、`--option=value` 形式と `--option value` 形式の両方を同等に受け付けなければならない。(MUST)

#### Scenario: `conv list` をスペース区切りで実行する
- Given `conv list --filter "is_private:true" --sort name --sort-dir desc --format table` を実行する
- When 会話一覧を取得してフィルタとソートを適用する
- Then `is_private:true` のチャンネルのみが降順で並び、`table` 形式で出力される
