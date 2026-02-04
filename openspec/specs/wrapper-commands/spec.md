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

### Requirement: msg post はスレッド返信を指定できる
`msg post` は `--thread-ts` が指定された場合に `chat.postMessage` へ `thread_ts` を渡さなければならない。(MUST)
#### Scenario: thread_ts を指定してスレッド返信を送る
- Given `--thread-ts` が指定されている
- When `msg post` を実行する
- Then `chat.postMessage` に `thread_ts` が渡される

### Requirement: reply_broadcast はスレッド返信のときのみ指定できる
`msg post` は `--reply-broadcast` が指定された場合に `reply_broadcast=true` を渡さなければならない。(MUST)
`msg post` は `--thread-ts` なしで `--reply-broadcast` が指定された場合にエラーで終了しなければならない。(MUST)
#### Scenario: reply_broadcast を指定してスレッド返信を送る
- Given `--thread-ts` と `--reply-broadcast` が指定されている
- When `msg post` を実行する
- Then `chat.postMessage` に `reply_broadcast=true` が渡される

### Requirement: Destructive operations require confirmation without `--yes`
`msg delete` MUST display confirmation if `--yes` is not present. (MUST)
#### Scenario: Execute `msg delete` without `--yes`
- Given executing `msg delete`
- When `--yes` is not specified
- Then confirmation is requested
