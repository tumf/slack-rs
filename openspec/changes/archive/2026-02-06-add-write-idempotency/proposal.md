## Summary
エージェント暴走による二重書き込みを防止するため、書き込み系ラッパーコマンドに冪等キーを導入する。`--idempotency-key` を指定した場合、初回実行結果をローカルに保存して再実行時は replay し、同一キーで内容が異なる場合は即時エラーとする。保存は TTL 7日 + 件数上限 + GC を標準とし、保管場所と権限を明示する。

## Scope
- `msg post/update/delete`, `react add/remove`, `file upload` に `--idempotency-key` を追加
- ローカル冪等ストアの追加（TTL 7日、上限件数、GC）
- エンベロープ meta に `idempotency_key` / `idempotency_status` を追加（キー指定時のみ）
- ヘルプ/イントロスペクションの更新

## Out of Scope
- Slack API へのサーバー側冪等キー対応
- `api call` のリトライ方針変更
- 既存の書き込み確認（`--yes` / `SLACKCLI_ALLOW_WRITE`）の仕様変更

## Motivation
非対話環境のエージェント実行では、再試行やループが起きやすく、`msg post` などの非冪等操作が二重投稿になりやすい。CLI 側で replay 可能な冪等性を提供し、意図しない二重書き込みを防ぐ。

## Risks / Tradeoffs
- ローカルストアにメッセージ本文などのレスポンスが保存されるため、保管場所と権限を厳格にする必要がある。
- 同一キーで内容変更を許可しないため、誤用時はエラーになる（fail fast）。

## Dependencies
- 追加の外部サービス依存なし
