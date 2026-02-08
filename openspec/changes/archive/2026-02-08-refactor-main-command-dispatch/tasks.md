# 実装タスク

- [x] `src/main.rs` のトップレベル `match args[1]` をコマンド単位ハンドラへ抽出する（検証: `cargo test --test commands_integration` が成功し、`main` から各ハンドラ呼び出し経路が確認できる）。
- [x] 重複しているエラー終了処理を共通ヘルパーへ抽出する（検証: `cargo test --test non_interactive_integration` が成功し、非対話時の終了コード期待が維持される）。
- [x] `auth` / `config` / `conv` のサブコマンド分岐を個別関数へ段階抽出する（検証: `cargo test --test auth_integration` と `cargo test --test oauth_integration` が成功する）。
- [x] `msg` / `react` / `file` の失敗時終了コード（通常 1、非対話エラー 2）を回帰防止テストで固定化する（検証: `cargo test --test commands_integration` と `cargo test --test file_download_integration` が成功する）。
- [x] ディスパッチ分解後にヘルプ系コマンド（`--help`, `commands --json`, `schema`）の出力互換性を確認する（検証: `cargo test --test output_envelope_tests` が成功し、手動確認で主要ヘルプ文言差分がないことを確認する）。

## 実装完了

すべてのタスクが完了しました。主な変更点:

1. **共通エラーハンドラの導入**: `handle_command_error` 関数を追加し、エラーメッセージの出力と終了コードの決定を一元化
2. **コマンドハンドラの抽出**: 以下のハンドラ関数を作成
   - `handle_auth_command`: auth サブコマンドのディスパッチ
   - `handle_config_command`: config サブコマンドのディスパッチ
   - `handle_conv_command`: conv サブコマンドのディスパッチ
   - `handle_users_command`: users サブコマンドのディスパッチ
   - `handle_msg_command`: msg サブコマンドのディスパッチ
   - `handle_react_command`: react サブコマンドのディスパッチ
   - `handle_file_command`: file サブコマンドのディスパッチ
3. **main 関数の簡素化**: トップレベルのコマンド分岐から各ハンドラへの委譲に変更し、責務を明確化

### 検証結果

- すべてのテストが成功（495 passed、unit + integration + doc tests）
- 終了コードの互換性維持（非対話エラーで exit 2、通常エラーで exit 1）
- ヘルプコマンド出力の互換性確認（`--help`, `commands --json`, `schema`）
- `cargo clippy -- -D warnings` 完全合格
- 既存の CLI 挙動を完全に保持
