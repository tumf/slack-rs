# wrapper-commands Specification

## Purpose
Provides user-friendly wrapper commands that abstract common Slack API operations with simplified interfaces and built-in safety mechanisms.
## Requirements
### Requirement: Search command enables message searching
The `search` command MUST call `search.messages` and pass `query`, `count`, `sort`, and `sort_dir` parameters.

#### Scenario: Execute search with query parameter
- Given valid profile and token exist
- When `search "invoice"` is executed
- Then `query` is passed to `search.messages`

### Requirement: Conv list retrieves conversation list
The `conv list` command MUST call `conversations.list` and pass `types` and `limit` parameters.

#### Scenario: Specify types and limit
- Given types and limit are specified
- When `conv list` is executed
- Then `types` and `limit` are passed to `conversations.list`

### Requirement: Conv history retrieves conversation history
The `conv history` command MUST call `conversations.history` and pass `channel`, `oldest`, `latest`, and `limit` parameters.

#### Scenario: Retrieve history by specifying channel
- Given channel id is specified
- When `conv history --channel C123` is executed
- Then `channel` is passed to `conversations.history`

### Requirement: Users info retrieves user information
The `users info` command MUST call `users.info` and pass the `user` parameter.

#### Scenario: Specify user id
- Given user id is specified
- When `users info --user U123` is executed
- Then `user` is passed to `users.info`

### Requirement: Msg command enables message operations
The `msg post/update/delete` commands MUST call `chat.postMessage` / `chat.update` / `chat.delete` respectively.

#### Scenario: Execute msg post
- Given channel and text are specified
- When `msg post` is executed
- Then `chat.postMessage` is called

### Requirement: React command enables reaction operations
The `react add/remove` commands MUST call `reactions.add` / `reactions.remove` respectively.

#### Scenario: Execute react add
- Given channel, ts, and emoji are specified
- When `react add` is executed
- Then `reactions.add` is called

### Requirement: Destructive operations require confirmation without --yes flag
The `msg delete` command MUST display confirmation when the `--yes` flag is not provided.

#### Scenario: Execute msg delete without --yes flag
- Given `msg delete` is executed
- When `--yes` flag is not specified
- Then confirmation is required

### Requirement: Write operations are controlled by environment variable
Write operations MUST determine permission/denial based on the `SLACKCLI_ALLOW_WRITE` environment variable value.
When `SLACKCLI_ALLOW_WRITE` is unset, write operations MUST be allowed.
The `--allow-write` flag MUST NOT be required, and if specified MUST NOT affect behavior.

#### Scenario: Execute msg post with SLACKCLI_ALLOW_WRITE unset
- Given executing a write operation
- When `SLACKCLI_ALLOW_WRITE` is unset
- Then write operation is allowed

#### Scenario: Execute msg post with SLACKCLI_ALLOW_WRITE=false
- Given executing a write operation
- When `SLACKCLI_ALLOW_WRITE` is set to `false` or `0`
- Then exit with error

#### Scenario: Execute msg post with --allow-write flag
- Given `SLACKCLI_ALLOW_WRITE` is unset
- When `--allow-write` flag is specified
- Then write operation is allowed

### Requirement: msg post supports thread replies
`msg post` MUST pass `thread_ts` to `chat.postMessage` when `--thread-ts` is specified. (MUST)
#### Scenario: Send thread reply with thread_ts
- Given `--thread-ts` is specified
- When executing `msg post`
- Then `thread_ts` is passed to `chat.postMessage`

### Requirement: reply_broadcast can only be specified with thread replies
`msg post` MUST pass `reply_broadcast=true` when `--reply-broadcast` is specified. (MUST)
`msg post` MUST exit with error when `--reply-broadcast` is specified without `--thread-ts`. (MUST)
#### Scenario: Send thread reply with reply_broadcast
- Given `--thread-ts` and `--reply-broadcast` are specified
- When executing `msg post`
- Then `reply_broadcast=true` is passed to `chat.postMessage`

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

