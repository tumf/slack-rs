# wrapper-commands Specification

## Purpose
Provides user-friendly wrapper commands that abstract common Slack API operations with simplified interfaces and built-in safety mechanisms.
## Requirements
### Requirement: search command can search messages
`search` MUST call `search.messages` and pass `query`, `count`, `sort`, and `sort_dir`. (MUST)
#### Scenario: Search by passing query
- Given a valid profile and token exist
- When executing `search "invoice"`
- Then `query` is passed to `search.messages`

### Requirement: conv list can retrieve conversation list
`conv list` MUST call `conversations.list` and pass `types` and `limit`. (MUST)
#### Scenario: Specify types and limit
- Given types and limit are specified
- When executing `conv list`
- Then `types` and `limit` are passed to `conversations.list`

### Requirement: conv history can retrieve history
`conv history` MUST call `conversations.history` and pass `channel`, `oldest`, `latest`, and `limit`. (MUST)
#### Scenario: Retrieve history by specifying channel
- Given channel id is specified
- When executing `conv history --channel C123`
- Then `channel` is passed to `conversations.history`

### Requirement: users info can retrieve user information
`users info` MUST call `users.info` and pass `user`. (MUST)
#### Scenario: Specify user id
- Given user id is specified
- When executing `users info --user U123`
- Then `user` is passed to `users.info`

### Requirement: msg command can manipulate messages
`msg post/update/delete` MUST call `chat.postMessage` / `chat.update` / `chat.delete`. (MUST)
#### Scenario: Execute msg post
- Given channel and text are specified
- When executing `msg post`
- Then `chat.postMessage` is called

### Requirement: react command can manipulate reactions
`react add/remove` MUST call `reactions.add` / `reactions.remove`. (MUST)
#### Scenario: Execute react add
- Given channel, ts, and emoji are specified
- When executing `react add`
- Then `reactions.add` is called

### Requirement: write operations require `--allow-write`
Write operations MUST be rejected if `--allow-write` is not present. (MUST)
#### Scenario: Execute `msg post` without `--allow-write`
- Given executing a write operation
- When `--allow-write` is not specified
- Then it exits with an error

### Requirement: Destructive operations require confirmation without `--yes`
`msg delete` MUST display confirmation if `--yes` is not present. (MUST)
#### Scenario: Execute `msg delete` without `--yes`
- Given executing `msg delete`
- When `--yes` is not specified
- Then confirmation is requested

### Requirement: file upload で外部アップロード方式を実行できる
`file upload` は `files.getUploadURLExternal` を呼び出し、取得した `upload_url` へファイルの生バイトを送信し、`files.completeUploadExternal` を呼び出して共有を完了しなければならない。(MUST)
`files.completeUploadExternal` には `files`（`id` と任意 `title`）を含め、`--channel`/`--channels`/`--comment` が指定されている場合は対応するパラメータを送信しなければならない。(MUST)
旧方式の `files.upload` を呼び出してはならない。(MUST NOT)

#### Scenario: channel を指定して file upload を実行する
- Given 有効な profile と token が存在する
- When `file upload ./report.pdf --allow-write --channel=C123 --comment="Weekly report"` を実行する
- Then `files.getUploadURLExternal` が `filename` と `length` 付きで呼ばれる
- And 返却された `upload_url` にファイルの生バイトが送信される
- And `files.completeUploadExternal` に `files` と `channel_id` と `initial_comment` が送信される

