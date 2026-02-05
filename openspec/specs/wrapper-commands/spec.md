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

### Requirement: Wrapper commands accept `--token-type`
Wrapper commands MUST accept `--token-type user|bot` and use the specified token. (MUST)
#### Scenario: Explicitly specify bot in conv list
- Given executing `conv list --token-type bot`
- When calling API
- Then Bot Token is used in Authorization header

### Requirement: Use default token type
`--token-type` が指定されていない場合、ラッパーコマンドはプロフィールの `default_token_type` を解決結果として扱わなければならない。(MUST)
`SLACK_TOKEN` が設定されている場合は、トークンソースとしてそれを優先し、token store の内容に関わらず `SLACK_TOKEN` を使用しなければならない。(MUST)

#### Scenario: default_token_type と SLACK_TOKEN の併用
- Given `default_token_type=user` が設定されている
- And `SLACK_TOKEN` が設定されている
- When `msg post` を実行する
- Then リクエストのトークンは `SLACK_TOKEN` が使用される
- And メタ情報上の `token_type` は `user` として扱われる

### Requirement: conv list にフィルタを追加する
`conv list` は `--filter` で `name:<glob>` と `is_member:true|false` と `is_private:true|false` を受け付け、AND 条件で絞り込むこと。 (MUST)
#### Scenario: 名前と参加状態で絞り込む
- Given `conv list --filter "name:ark*" --filter "is_member:true"` を実行する
- When 会話一覧を取得する
- Then 条件に一致するチャンネルのみが出力される

### Requirement: conv select を提供する
`conv select` は会話一覧を対話表示し、選択したチャンネル ID を返すこと。 (MUST)
#### Scenario: 対話選択でチャンネル ID を取得する
- Given `conv select` を実行する
- When 一覧からチャンネルを選択する
- Then 選択したチャンネル ID が出力される

### Requirement: conv history にインタラクティブ選択を追加する
`conv history --interactive` は会話一覧からチャンネルを選び、その ID で履歴取得を行うこと。 (MUST)
#### Scenario: チャンネル選択後に履歴取得する
- Given `conv history --interactive` を実行する
- When 対話でチャンネルを選択する
- Then 選択したチャンネルの履歴が取得される

### Requirement: Display guidance when private_channel retrieval fails
When `types=private_channel` is specified in `conv list`, guidance MUST be displayed if the result is empty and Bot Token is being used. (MUST)
#### Scenario: Empty private channels with Bot Token
- Given executing `conv list --types private_channel` using Bot Token
- When the result is empty
- Then guidance about using User Token or inviting the bot is displayed

### Requirement: Explicit error when User Token does not exist
When `private_channel` is requested without User Token available, an error MUST be returned indicating User Token is required. (MUST)
#### Scenario: User Token unavailable
- Given User Token is not stored
- When executing `conv list --types private_channel`
- Then an error indicating User Token is required is returned

### Requirement: Wrapper commands output is normalized
ラッパーコマンドの JSON 出力は `response`/`meta` のエンベロープで返し、`meta.profile_name`, `meta.team_id`, `meta.user_id`, `meta.method`, `meta.command` を含むこと。`--raw` が指定された場合は Slack API レスポンスをそのまま返すこと。(MUST)

#### Scenario: `conv list` は統一フォーマットで返る
- Given 有効な profile と token が存在する
- When `conv list` を実行する
- Then `meta.command` は `conv list` である
- And `meta.method` は `conversations.list` である
- And `response` に Slack API レスポンスが入る

#### Scenario: `--raw` 指定時は Slack API レスポンスを返す
- Given 有効な profile と token が存在する
- When `conv list --raw` を実行する
- Then 出力は Slack API レスポンスの JSON そのままである
- And `meta` フィールドは含まれない

### Requirement: Wrapper commands show guidance for known Slack error codes
ラッパーコマンドの実行結果が `ok=false` かつ `error` が既知のコードに一致する場合、標準エラー出力に `Error:`/`Cause:`/`Resolution:` を含むガイダンスを表示しなければならない。(MUST)

#### Scenario: `missing_scope` のガイダンスが表示される
- Given Slack API が `ok=false` と `error=missing_scope` を返す
- When `users info --user U123` を実行する
- Then 標準エラー出力に `Error:`/`Cause:`/`Resolution:` が含まれる
- And JSON 出力は Slack のレスポンスのままである

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

### Requirement: conv command behavior is preserved after internal module split
`conv list`/`conv select`/`conv history` MUST maintain existing argument, output, and error behavior even after internal restructuring. (MUST)

#### Scenario: Behavior is preserved after module split
- Given using existing `conv` command arguments
- When executing `conv list` or `conv history`
- Then the same API calls and output format as before are maintained

### Requirement: Write operation help text indicates environment variable
Write operation usage/help text MUST clearly indicate control via `SLACKCLI_ALLOW_WRITE` and MUST NOT mislead users into thinking `--allow-write` is required. (MUST)

#### Scenario: Help display shows write control mechanism
- Given displaying `slack-rs` help/usage
- When checking description of write operations like `msg`/`react`
- Then control via `SLACKCLI_ALLOW_WRITE` is indicated
- And no required notation like `requires --allow-write` is included

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

### Requirement: `conv list` accepts space-separated options
Value-taking options for `conv list` (`--filter`/`--format`/`--sort`/`--sort-dir`/`--types`/`--limit`/`--profile`) MUST accept both `--option=value` and `--option value` formats equivalently. (MUST)

#### Scenario: Execute `conv list` with space-separated options
- Given executing `conv list --filter "is_private:true" --sort name --sort-dir desc --format table`
- When retrieving conversation list and applying filter and sort
- Then only channels matching `is_private:true` are output in descending order in `table` format

