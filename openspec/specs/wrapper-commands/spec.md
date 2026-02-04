# wrapper-commands Specification

## Purpose
Provides high-level wrapper commands for common Slack API operations, enabling quick message searches, conversation management, user information retrieval, and message/reaction operations without manually constructing API calls.

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

