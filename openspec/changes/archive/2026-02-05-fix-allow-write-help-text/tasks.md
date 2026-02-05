- [x] `src/main.rs` の usage 文言を `SLACKCLI_ALLOW_WRITE` に合わせて修正する（検証: `src/main.rs` の `print_usage` に `requires SLACKCLI_ALLOW_WRITE=true` が表示され、`requires --allow-write` が消えていることを確認）
- [x] ヘルプ表示の最終確認を行う（検証: `cargo run -- --help` もしくは `slack-rs --help` の出力に `SLACKCLI_ALLOW_WRITE` が表示されることを確認）

## Acceptance #1 Failure Follow-up
- [x] `slack-rs --help` が呼び出す `src/main.rs` の `print_help` に write 操作の制御が `SLACKCLI_ALLOW_WRITE` である旨の記載を追加する（現在は `msg`/`react` の説明に環境変数の記載がない）
- [x] `slack-rs msg`/`slack-rs react` の使用時に出る `src/cli/mod.rs` の `print_msg_usage` と `print_react_usage` に `SLACKCLI_ALLOW_WRITE` による制御を明示する

## Verification
- [x] ビルドが成功することを確認する（cargo build）
- [x] テストが通ることを確認する（cargo test）
- [x] `cargo run -- --help` の出力で `SLACKCLI_ALLOW_WRITE` の記載を確認する
- [x] `cargo run -- msg` と `cargo run -- react` の使用時ヘルプで `SLACKCLI_ALLOW_WRITE` の記載を確認する
