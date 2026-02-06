- [x] 共通の profile 解決ヘルパーを追加し、`--profile` > `SLACK_PROFILE` > `default` の順で解決することを確認する（確認: 新ヘルパーの単体テスト、または該当関数のロジック確認）
- [x] `api call` の profile 解決を共通ヘルパーに置き換える（確認: `src/cli/handlers.rs` で `SLACK_PROFILE` 直参照が無くなる）
- [x] wrapper コマンド群の profile 解決に `SLACK_PROFILE` フォールバックを追加する（確認: `src/cli/mod.rs` の各 `--profile` 解決が共通ヘルパー経由になる）
- [x] `--profile <name>` と `--profile=<name>` が前置/後置どちらでも認識されることを確認する（確認: ルーティング前の引数正規化または解析のテスト追加）

## Acceptance #1 Failure Follow-up
- [x] `--profile` をコマンド前に置いた場合にルーティングできないため、グローバルフラグ前置きを許容する前処理（または引数正規化）を `src/main.rs` で追加する
- [x] `api call` で `--profile <name>` のスペース区切りが `ApiCallArgs::parse` によって `InvalidKeyValue` になるため、`--profile` を解析前に除去/正規化してスペース形式も受け付けるようにする
- [x] `file upload` が `resolve_profile_name` を使わず `SLACK_PROFILE` のフォールバックを無視しているため、共通ヘルパー経由に修正する
