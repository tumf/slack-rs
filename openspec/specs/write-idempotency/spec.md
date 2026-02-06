# write-idempotency Specification

## Purpose
TBD - created by archiving change add-write-idempotency. Update Purpose after archive.
## Requirements
### Requirement: 書き込み操作は `--idempotency-key` で replay 可能にする
書き込み系ラッパーコマンドは `--idempotency-key` を受け付け、同一キーでの再実行時は Slack API を再呼び出しせず、保存済みレスポンスを返さなければならない。(MUST)
同一キーでリクエスト内容が異なる場合は即時エラーにし、Slack API を呼び出してはならない。(MUST NOT)
冪等キーは `team_id`, `user_id`, `method` を含むスコープで保存し、衝突を避けなければならない。(MUST)

#### Scenario: `msg post` を同一キーで再実行する
- Given `msg post` が `--idempotency-key deploy-123` で一度成功している
- When 同じ引数で再実行する
- Then Slack API を再呼び出しせずに保存済みレスポンスが返る

### Requirement: 冪等ストアは TTL 7日 + 件数上限 + GC を行う
冪等ストアはエントリに TTL 7日を設定し、期限切れを GC で削除しなければならない。(MUST)
冪等ストアは上限件数 10,000 を超える場合、古い順に削除して上限内に収めなければならない。(MUST)
GC はストア読み込み時または書き込み前に必ず実行しなければならない。(MUST)

#### Scenario: 期限切れと上限超過を GC する
- Given 冪等ストアに期限切れエントリが存在する
- And 冪等ストアの件数が上限を超過している
- When `msg post --idempotency-key` を実行してストアを読み込む
- Then 期限切れエントリが削除される
- And 上限を超える古いエントリが削除される

### Requirement: 冪等ストアは config dir に保存し Unix では 0600 を維持する
冪等ストアは OS の config dir 配下に保存し、デフォルトで `~/.config/slack-rs/idempotency_store.json` を使用しなければならない。(MUST)
Unix 環境ではファイル権限を 0600 に設定しなければならない。(MUST)

#### Scenario: 冪等ストアの保存先と権限を確認する
- Given Unix 環境で冪等ストアを新規作成する
- When 1件のエントリを保存する
- Then config dir 配下にファイルが作成される
- And ファイル権限が 0600 である

