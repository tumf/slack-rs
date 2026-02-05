# タスク

- [x] `src/commands/conv/` 配下に責務別モジュールを作成する（確認: `filter.rs`/`sort.rs`/`format.rs`/`api.rs`/`select.rs` が存在する）
- [x] 既存の `conv` 関連関数を対応するモジュールへ移動する（確認: `src/commands/conv/mod.rs` から再公開されている）
- [x] 参照箇所の `use`/`mod` 宣言を更新する（確認: `cargo check` が通る）
- [x] `conv` 系の挙動回帰を防ぐためテストを実行する（確認: `cargo test commands_integration` または `cargo test`）
