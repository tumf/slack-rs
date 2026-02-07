## MODIFIED Requirements

### Requirement: Conv list retrieves conversation list
`conv list` MUST accept `--include-private` and `--all` and reflect them in `types` resolution. (MUST)
When both `--types` and `--include-private`/`--all` are specified simultaneously, an error MUST be returned. (MUST)
When none of `--types`, `--include-private`, `--all` are specified, `conv list` MUST use `types=public_channel,private_channel` as default. (MUST)
When `--limit` is omitted, `conv list` MUST use `limit=1000` and follow `response_metadata.next_cursor` until exhausted, merging all pages before downstream filtering/formatting. (MUST)

#### Scenario: Default conv list includes private channels
- Given `--types`, `--include-private`, and `--all` are not specified
- When executing `conv list`
- Then `public_channel,private_channel` is passed to `types`

#### Scenario: Default conv list follows pagination
- Given `conversations.list` response has `response_metadata.next_cursor`
- When executing `conv list` without `--limit`
- Then the command requests subsequent pages until cursor is empty
- And channels from all pages are merged in the final result

#### Scenario: Explicit --types keeps priority
- Given `--types=public_channel` is specified
- When executing `conv list --types=public_channel`
- Then only the explicit `types` value is used
- And default `public_channel,private_channel` is not applied

### Requirement: `conv search` は名前で検索できる
`conv search <pattern>` は `conversations.list` の結果に対して `name:<pattern>` のフィルタを適用し、結果を出力しなければならない。(MUST)
`conv search` は `--types`/`--limit`/`--format`/`--sort`/`--sort-dir` を `conv list` と同様に受け付けなければならない。(MUST)
`conv search` で `--types` と `--limit` が未指定の場合、会話取得の既定値（`types=public_channel,private_channel`, `limit=1000`, `next_cursor` 追従）を `conv list` と同一に適用しなければならない。(MUST)

#### Scenario: `conv search` default can discover private channel on later page
- Given 対象 private channel が 2 ページ目以降に存在する
- And `conv search` 実行時に `--types` と `--limit` は未指定である
- When `conv search "infra"` を実行する
- Then `conversations.list` は cursor を辿って複数ページ取得される
- And private channel が検索結果に含まれる
