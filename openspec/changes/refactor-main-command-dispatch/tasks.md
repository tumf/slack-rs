# 実装タスク

- [ ] `src/main.rs` のトップレベル `match args[1]` をコマンド単位ハンドラへ抽出する（検証: `cargo test --test commands_integration` が成功し、`main` から各ハンドラ呼び出し経路が確認できる）。
- [ ] 重複しているエラー終了処理を共通ヘルパーへ抽出する（検証: `cargo test --test non_interactive_integration` が成功し、非対話時の終了コード期待が維持される）。
- [ ] `auth` / `config` / `conv` のサブコマンド分岐を個別関数へ段階抽出する（検証: `cargo test --test auth_integration` と `cargo test --test oauth_integration` が成功する）。
- [ ] `msg` / `react` / `file` の失敗時終了コード（通常 1、非対話エラー 2）を回帰防止テストで固定化する（検証: `cargo test --test commands_integration` と `cargo test --test file_download_integration` が成功する）。
- [ ] ディスパッチ分解後にヘルプ系コマンド（`--help`, `commands --json`, `schema`）の出力互換性を確認する（検証: `cargo test --test output_envelope_tests` が成功し、手動確認で主要ヘルプ文言差分がないことを確認する）。
