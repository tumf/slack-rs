# wrapper-commands Specification (Delta)

## MODIFIED Requirements

### Requirement: Conv list retrieves conversation list
`conv list` は `--include-private` と `--all` を受け付け、`types` の解決に反映しなければならない。(MUST)
`--types` と `--include-private`/`--all` が同時に指定された場合はエラーにしなければならない。(MUST)

#### Scenario: `--include-private` を指定する
- Given `--types` が指定されていない
- When `conv list --include-private` を実行する
- Then `types` に `public_channel,private_channel` が渡される

#### Scenario: `--all` を指定する
- Given `--types` が指定されていない
- When `conv list --all` を実行する
- Then `types` に `public_channel,private_channel,im,mpim` が渡される

#### Scenario: `--types` と `--all` を併用する
- Given `--types=public_channel` が指定されている
- When `conv list --types=public_channel --all` を実行する
- Then `--types` と `--all` の併用はエラーになる

### Requirement: Conv search matches are user-friendly by default
`conv search` の検索はデフォルトで大小無視の部分一致でなければならない。(MUST)
パターンに `*` が含まれる場合は glob マッチを使用しなければならない。(MUST)

#### Scenario: 大小無視の部分一致で検索する
- When `conv search Gen` を実行する
- Then `general` が一致候補として扱われる

#### Scenario: `*` を含む場合は glob マッチを使う
- When `conv search "gen*"` を実行する
- Then `gen` から始まるチャンネル名が一致する

### Requirement: Wrapper commands show guidance for known Slack error codes
`channel_not_found` が返却された場合、標準エラー出力に原因候補と次のアクションを表示しなければならない。(MUST)

#### Scenario: `channel_not_found` のガイダンスが表示される
- Given Slack API が `ok=false` と `error=channel_not_found` を返す
- When `conv history C123` を実行する
- Then stderr に「private で未参加」「token type」「profile」の可能性と確認方法が表示される
