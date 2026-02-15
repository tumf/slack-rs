# thread get コマンド

## ADDED Requirements

### Requirement: `thread get` は `conversations.replies` をラップしてスレッドを取得できる

`thread get` MUST 指定されたチャンネルとスレッドタイムスタンプを用いて `conversations.replies` を実行し、対象スレッドのメッセージを取得する。

#### Scenario: channel と thread_ts を指定してスレッドを取得する
Given `slack-rs thread get C123456 1700000000.000001` を実行する
When CLI が `conversations.replies` を呼び出す
Then `channel=C123456` と `ts=1700000000.000001` が送信される

### Requirement: `thread get` は `--limit` と `--inclusive` を API パラメータとして渡す

`--limit` と `--inclusive` の指定内容 MUST `conversations.replies` のリクエストパラメータに反映される。

#### Scenario: `--limit` と `--inclusive` を指定して取得する
Given `slack-rs thread get C123456 1700000000.000001 --limit=50 --inclusive` を実行する
When CLI が `conversations.replies` を呼び出す
Then `limit=50` と `inclusive=true` が送信される

### Requirement: `thread get` はカーソルページネーションを追従し messages を集約する

`response_metadata.next_cursor` が返る場合、CLI MUST 追加ページを取得し、全ページの `messages` を結合して返す。

#### Scenario: `next_cursor` が返る場合に複数ページを統合する
Given 1 ページ目のレスポンスに `response_metadata.next_cursor` が含まれる
When CLI が追加ページを取得する
Then `messages` は全ページ分の配列として返される

### Requirement: `thread get` は統一エンベロープを既定出力し `--raw` で生レスポンスを返す

既定では統一エンベロープを出力し、`--raw` が指定された場合は Slack API の生レスポンスを出力することを CLI MUST 守る。

#### Scenario: `--raw` なしと `--raw` 指定で出力形式が切り替わる
Given `slack-rs thread get C123456 1700000000.000001` を実行する
When `--raw` を指定しない
Then `response` と `meta` を含む統一エンベロープが出力される
When `--raw` を指定する
Then Slack API の生レスポンスが出力される

### Requirement: `thread get` はヘルプとイントロスペクションに表示される

`commands --json` と `--help --json` の結果に `thread get` が含まれることを CLI MUST 保証する。

#### Scenario: `commands --json` と `--help --json` に `thread get` が含まれる
Given `slack-rs commands --json` を実行する
When コマンド一覧を取得する
Then `thread get` が含まれる
