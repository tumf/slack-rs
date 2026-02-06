- [x] 冪等ストアのデータ構造と JSON 永続化を追加する（保存先: config dir、TTL 7日、上限 10,000、GC）。完了確認: ユニットテストで期限切れ削除と上限削除が通ること。
- [x] 冪等キーのスコープ化（team_id/user_id/method）と request fingerprint 検証を実装する。完了確認: 同一キーで内容が異なる場合にエラーになるテストが通ること。
- [x] write 操作（`msg post/update/delete`, `react add/remove`, `file upload`）に `--idempotency-key` を追加し、replay 時に Slack API を呼び出さないように配線する。完了確認: スタブ化したクライアントで replay が呼び出し抑止されるテストが通ること。
- [x] エンベロープ meta に `idempotency_key` と `idempotency_status` を追加し、キー指定時のみ出力する。完了確認: `--idempotency-key` 付き実行時の JSON に meta が含まれることをテストで確認する。
- [x] write 操作のヘルプ/イントロスペクション出力に `--idempotency-key` を追加する。完了確認: `--help --json` またはヘルプ文字列でフラグが表示されることを確認する。
- [x] 追加テストの実行（`cargo test` の関連テスト）。完了確認: 追加分のテストが成功すること。

## Acceptance #1 Failure Follow-up
- [x] `print_msg_usage` のヘルプ/使用例に `--idempotency-key` を追加する。
- [x] `print_react_usage` のヘルプ/使用例に `--idempotency-key` を追加する。
- [x] `print_file_usage` のヘルプ/使用例に `--idempotency-key` を追加する。
