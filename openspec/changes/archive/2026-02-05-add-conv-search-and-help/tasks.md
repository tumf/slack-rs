- [x] `conv search` コマンドの引数解析と実行パスを追加する（確認: `slack-rs conv search <pattern>` が実行できる）
- [x] `conv search` に `name:<pattern>` フィルタを注入して `conv list` と同じ出力形式を返す（確認: 既存のフィルタ適用ロジックで結果が絞り込まれる）
- [x] `conv search --select` を実装し、選択されたチャンネル ID のみを出力する（確認: 対話選択の結果が標準出力に出る）
- [x] `conv list --help` に `--filter`/`--format`/`--sort`/`--sort-dir` の説明を追加する（確認: ヘルプ出力に各項目が含まれる）
- [x] 追加機能のテストを追加する（確認: フィルタ適用と選択ロジックのユニットテストが通る）

## Acceptance #1 Failure Follow-up
- [x] `conv list --help` がヘルプを表示せず API を呼び出すため、`--help` を検出して `print_conv_usage` を表示するようにする（`src/main.rs`/`src/cli/mod.rs`）。
