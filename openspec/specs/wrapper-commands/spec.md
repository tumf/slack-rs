# wrapper-commands Specification

## Purpose
TBD - created by archiving change add-wrapper-commands. Update Purpose after archive.
## Requirements
### Requirement: search コマンドでメッセージ検索ができる
`search` は `search.messages` を呼び出し、`query`, `count`, `sort`, `sort_dir` を渡さなければならない。(MUST)
#### Scenario: query を渡して検索する
- Given 有効な profile と token が存在する
- When `search "invoice"` を実行する
- Then `search.messages` に `query` が渡される

### Requirement: conv list で会話一覧を取得できる
`conv list` は `conversations.list` を呼び出し、`types` と `limit` を渡さなければならない。(MUST)
#### Scenario: types と limit を指定する
- Given types と limit が指定されている
- When `conv list` を実行する
- Then `conversations.list` に `types` と `limit` が渡される

### Requirement: conv history で履歴を取得できる
`conv history` は `conversations.history` を呼び出し、`channel`, `oldest`, `latest`, `limit` を渡さなければならない。(MUST)
#### Scenario: channel を指定して履歴を取得する
- Given channel id が指定されている
- When `conv history --channel C123` を実行する
- Then `conversations.history` に `channel` が渡される

### Requirement: users info でユーザ情報を取得できる
`users info` は `users.info` を呼び出し、`user` を渡さなければならない。(MUST)
#### Scenario: user id を指定する
- Given user id が指定されている
- When `users info --user U123` を実行する
- Then `users.info` に `user` が渡される

### Requirement: msg コマンドでメッセージ操作ができる
`msg post/update/delete` は `chat.postMessage` / `chat.update` / `chat.delete` を呼び出さなければならない。(MUST)
#### Scenario: msg post を実行する
- Given channel と text が指定されている
- When `msg post` を実行する
- Then `chat.postMessage` が呼ばれる

### Requirement: react コマンドでリアクション操作ができる
`react add/remove` は `reactions.add` / `reactions.remove` を呼び出さなければならない。(MUST)
#### Scenario: react add を実行する
- Given channel と ts と emoji が指定されている
- When `react add` を実行する
- Then `reactions.add` が呼ばれる

### Requirement: write 操作は `--allow-write` が必須
write 操作は `--allow-write` が無い場合に拒否されなければならない。(MUST)
#### Scenario: `msg post` を `--allow-write` なしで実行する
- Given write 操作を実行する
- When `--allow-write` が指定されていない
- Then エラーで終了する

### Requirement: 破壊操作は `--yes` なしだと確認が入る
`msg delete` は `--yes` が無い場合に確認を表示しなければならない。(MUST)
#### Scenario: `msg delete` を `--yes` なしで実行する
- Given `msg delete` を実行する
- When `--yes` が指定されていない
- Then 確認が求められる

