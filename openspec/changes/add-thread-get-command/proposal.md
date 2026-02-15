## 変更提案: thread get コマンド追加

### 背景
Slack のスレッド取得は `conversations.replies` が必要ですが、現状は `api call` や curl が前提です。CLI でスレッドを直接取得できる first-class コマンドが求められています。

### 目的
- `slack-rs thread get <channel_id> <thread_ts>` でスレッドを取得できる
- 既存の wrapper コマンドと同様に統一エンベロープを既定出力とする
- `--raw` で Slack API の生レスポンスを返す
- カーソルページネーションを追従し、スレッド全体を取得できる

### スコープ
- `thread get` コマンドの追加（引数: channel, thread_ts, --limit, --inclusive, --raw, --profile, --token-type）
- `conversations.replies` を呼び出す wrapper 実装
- ヘルプ表示・コマンド一覧・構造化ヘルプへの反映

### 非対象
- スレッド取得結果の整形（表形式・tsv など）
- スレッド取得結果の保存やファイル出力
- リアル Slack 環境を必要とする統合テスト

### 影響/リスク
- `conversations.replies` の token 種別とスコープ制約により、ユーザートークンが必要なケースがある
- ページネーション追従により複数リクエストが発生するためレート制限に注意が必要

### 受け入れ基準
- `thread get` が `conversations.replies` を呼び出しスレッドを取得できる
- `--raw` でエンベロープ無し、既定でエンベロープ有りの JSON が返る
- `next_cursor` がある場合に複数ページが結合される
- help/commands/introspection に `thread get` が反映される
