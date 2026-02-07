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
write 操作（`msg post/update/delete`, `react add/remove`, `file upload`）はデフォルトで確認を求めなければならない。(MUST)
`--yes` が指定されている場合は確認を省略しなければならない。(MUST)
非対話モードでは `--yes` が無い場合に即時エラーにしなければならない。(MUST)
write 操作のヘルプ/使用例には `--idempotency-key` の説明を含めなければならない。(MUST)

#### Scenario: ヘルプに `--idempotency-key` が表示される
- Given write 操作のヘルプ表示を確認する
- When `msg post --help` を表示する
- Then `--idempotency-key` の説明が含まれる

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
`SLACKRS_OUTPUT=raw` が設定されている場合、`--raw` が指定されていなくても raw 出力を既定とすること。(MUST)

#### Scenario: `SLACKRS_OUTPUT=raw` で既定出力が raw になる
- Given `SLACKRS_OUTPUT=raw` が設定されている
- And `--raw` が指定されていない
- When `conv list` を実行する
- Then 出力は Slack API レスポンスの JSON そのままである
- And `meta` フィールドは含まれない

### Requirement: Wrapper commands show guidance for known Slack error codes
When `channel_not_found` is returned, cause candidates and next actions MUST be displayed to stderr. (MUST)

#### Scenario: `channel_not_found` guidance is displayed
- Given Slack API returns `ok=false` and `error=channel_not_found`
- When executing `conv history C123`
- Then stderr displays possibilities of "private and not joined", "token type", and "profile" with verification methods

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
`conv search` で `--types` と `--limit` が未指定の場合、会話取得の既定値（`types=public_channel,private_channel`, `limit=1000`, `next_cursor` 追従）を `conv list` と同一に適用しなければならない。(MUST)

#### Scenario: `conv search` default can discover private channel on later page
- Given 対象 private channel が 2 ページ目以降に存在する
- And `conv search` 実行時に `--types` と `--limit` は未指定である
- When `conv search "infra"` を実行する
- Then `conversations.list` は cursor を辿って複数ページ取得される
- And private channel が検索結果に含まれる

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

### Requirement: Conv search matches are user-friendly by default
`conv search` MUST use case-insensitive substring matching by default. (MUST)
When the pattern contains `*`, glob matching MUST be used. (MUST)

#### Scenario: Search with case-insensitive substring matching
- When executing `conv search Gen`
- Then `general` is treated as a matching candidate

#### Scenario: Use glob matching when `*` is present
- When executing `conv search "gen*"`
- Then channel names starting with `gen` are matched

### Requirement: `file download` retrieves Slack files with authentication
`file download <file_id>` MUST call `files.info` to resolve the download URL and retrieve the file via authenticated GET. (MUST)
`file download <file_id>` MUST prefer `url_private_download` and fall back to `url_private` when unavailable. (MUST)
`file download --url <url_private_or_download>` MUST skip `files.info` and directly retrieve from the specified URL. (MUST)
`file download` の HTTP 取得は、Slack の標準的な配布経路に対応するため 3xx リダイレクトを追従しなければならない。(MUST)

#### Scenario: 3xx リダイレクト先を追従してダウンロードする
- Given 有効なトークンが存在し、初回 URL が 302 を返す
- When `file download F1234567890` を実行する
- Then クライアントは `Location` を追従し、最終到達先で認証付き GET を完了する
- And 最終応答が 2xx の場合はダウンロード成功として扱う

### Requirement: `file download` controls output destination and method
`file download` MUST accept `--out <path>` and write downloaded content to the specified destination. (MUST)
When `--out` is not specified, it MUST save to a safe default filename in the current directory. (MUST)
When `--out -` is specified, it MUST stream binary content to stdout and MUST NOT output any non-data content to stdout. (MUST NOT)

#### Scenario: Stream to stdout with `--out -`
- Given executing `file download F123 --out -`
- When download succeeds
- Then file bytes are written to stdout
- And progress or diagnostic messages do not mix with stdout

### Requirement: `file download` explicitly errors on HTML responses and HTTP failures
`file download` MUST exit with non-zero status and return a concise error message when the download response is non-2xx. (MUST)
`file download` MUST return an error indicating a possible URL mismatch or authentication issue when the download response has `Content-Type: text/html`. (MUST)
`Content-Type: text/html` の失敗時は、診断を容易にするためレスポンス本文の短い先頭断片（安全に切り詰めたスニペット）をエラーメッセージ文脈に含めなければならない。(MUST)

#### Scenario: HTML 応答時にヒントと本文スニペットを表示して失敗する
- Given ダウンロード先が `Content-Type: text/html` と短い HTML 本文を返す
- When `file download F123` を実行する
- Then コマンドは失敗終了する
- And stderr には URL 種別不一致または認証問題のヒントが含まれる
- And stderr には本文先頭の短いスニペットが含まれる

### Requirement: `file download` is excluded from write guard
`file download` MUST be treated as a read operation and MUST be executable even when `SLACKCLI_ALLOW_WRITE` is `false` or `0`. (MUST)

#### Scenario: Download is allowed with `SLACKCLI_ALLOW_WRITE=false`
- Given `SLACKCLI_ALLOW_WRITE=false` is set
- When executing `file download F123`
- Then no write-guard rejection occurs

