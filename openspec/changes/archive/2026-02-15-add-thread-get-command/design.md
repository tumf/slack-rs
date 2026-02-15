## 設計方針

### 全体像
`thread get` は `conversations.replies` の薄いラッパーとして実装する。既存の wrapper コマンドと同じ入出力・デバッグ・ガイダンスの流儀に合わせ、統一エンベロープを既定出力とする。

### CLI インターフェース
- 形式: `slack-rs thread get <channel_id> <thread_ts> [--limit=N] [--inclusive] [--raw] [--profile=NAME] [--token-type=bot|user]`
- `--limit` は 1 リクエストあたりのページサイズとして `conversations.replies` に渡す
- `--inclusive` は `oldest`/`latest` を指定しない場合は影響しないが、将来的な拡張を見据えて引数として受け付ける

### API 呼び出し
- `ApiMethod::ConversationsReplies` を追加し GET で呼び出す
- 必須引数: `channel`, `ts`
- 任意引数: `limit`, `inclusive`, `cursor`

### ページネーション戦略
- `response_metadata.next_cursor` を使って全ページを追従
- 各ページの `messages` を連結し、最終レスポンスの `messages` に集約結果を設定
- `response_metadata` は最終ページの内容で上書きし、`next_cursor` が空の状態で返す
- 無限ループ回避のため `next_cursor` の重複検知（HashSet）または `max_pages` ガードを設け、異常時はエラーにする

### 出力
- 既定: unified envelope (`CommandResponse`) で `type=conversations.replies`, `command=thread get`
- `--raw` または `SLACKRS_OUTPUT=raw` のときは Slack API の生レスポンス JSON を返す

### デバッグ/ガイダンス
- `--debug`/`--trace` のログは既存の `conv history` と同様に `debug::log_api_context` を使用
- `missing_scope`/`not_allowed_token_type` など既存のガイダンスを流用

### テスト方針（外部依存のモック化）
- 実 Slack への依存は避け、`wiremock`/`httpmock` を用いたモックサーバで `conversations.replies` を再現
- `ApiClient::new_with_base_url` を使い、ページネーションとパラメータ送信を検証する
