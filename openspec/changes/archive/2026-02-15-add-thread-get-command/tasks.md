## 1. API とコマンド層の追加

- [x] 1.1 `ApiMethod::ConversationsReplies` を追加し `as_str`/`uses_get_method` を更新する（検証: `src/api/types.rs` と `src/api/client.rs` のテストに `conversations.replies` が含まれる）
- [x] 1.2 `commands::thread_get` を新規実装し `conversations.replies` のページネーション集約を行う（検証: `src/commands/thread.rs` に `thread_get` とカーソル追従が実装されている）
- [x] 1.3 モックサーバで `thread_get` のパラメータ送信とページネーションを検証するテストを追加する（検証: `cargo test thread_get` がローカルで完走する）

## 2. CLI への配線

- [x] 2.1 `run_thread_get` を追加し引数解析・デバッグログ・出力（raw/envelope）を実装する（検証: `src/cli/mod.rs` に `run_thread_get` と `wrap_with_envelope_and_token_type` 呼び出しがある）
- [x] 2.2 `main.rs` に `thread` サブコマンドのディスパッチと使用例の表示を追加する（検証: `src/main.rs` の `print_help` と `print_usage` に `thread get` が含まれる）
- [x] 2.3 `print_thread_usage` を追加して `thread` コマンドのヘルプを提供する（検証: `src/cli/mod.rs` に `print_thread_usage` が追加され、`handle_thread_command` から参照される）

## 3. イントロスペクション更新

- [x] 3.1 `commands --json`/`--help --json` に `thread get` を追加する（検証: `src/cli/introspection.rs` に `thread get` の `CommandDef` が存在する）

## 4. 受け入れ前フォローアップ

- [x] 4.1 `git status --porcelain` をクリーンにする（検証: `git status --porcelain` が空を返す）
