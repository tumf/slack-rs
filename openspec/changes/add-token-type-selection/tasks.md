- [x] トークン種別の列挙型と解決ロジックを追加し、`--token-type` と `default_token_type` の優先順位が設計通りに動作することをユニットテストで確認する（例: `cargo test token_type_resolution`）。
- [x] `profiles.json` に `default_token_type` を保存・読み込みできるようにし、既存プロファイルとの後方互換性が維持されることをテストで確認する（例: 旧形式 JSON の読み込みテスト）。
- [x] `config` コマンドに既定トークン種別の設定を追加し、設定後に `profiles.json` に反映されることを確認する（例: `config set default --token-type user` の実行結果を検証）。
- [x] `api call` で `--token-type` と既定値に基づくトークン選択が行われ、Authorization ヘッダに反映されることを HTTP モックで確認する（例: `httpmock` でヘッダ検証）。
- [x] `api call` 出力の `meta.token_type` が選択された種別と一致することをテストで確認する（例: JSON 出力の検証）。
- [x] ラッパーコマンドが `--token-type` を受け付け、選択結果が実際の API 呼び出しに反映されることをテストで確認する（例: conv list のヘッダ検証）。

## Acceptance #1 Failure Follow-up
- [x] `api call --token-type user` で user トークンが存在しない場合に bot へフォールバックせず、明確なエラーで失敗するようにする（`src/main.rs` の `run_api_call` でのフォールバックを削除・修正）。
- [x] user トークンのキー形式を統一し、`api call` が `team_id:user_id:user` 形式で保存済みトークンを参照できるようにする（`src/main.rs` と `src/auth/commands.rs` のキー生成を一致させる）。
- [x] ラッパーコマンドで `--token-type` を受け付け、未指定時は `default_token_type` を用いてトークン解決する（`src/cli/mod.rs` の引数解析と `get_api_client` を更新）。
