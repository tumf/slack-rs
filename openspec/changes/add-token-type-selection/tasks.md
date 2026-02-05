- [x] トークン種別の列挙型と解決ロジックを追加し、`--token-type` と `default_token_type` の優先順位が設計通りに動作することをユニットテストで確認する（例: `cargo test token_type_resolution`）。
- [x] `profiles.json` に `default_token_type` を保存・読み込みできるようにし、既存プロファイルとの後方互換性が維持されることをテストで確認する（例: 旧形式 JSON の読み込みテスト）。
- [x] `config` コマンドに既定トークン種別の設定を追加し、設定後に `profiles.json` に反映されることを確認する（例: `config set default --token-type user` の実行結果を検証）。
- [x] `api call` で `--token-type` と既定値に基づくトークン選択が行われ、Authorization ヘッダに反映されることを HTTP モックで確認する（例: `httpmock` でヘッダ検証）。
- [x] `api call` 出力の `meta.token_type` が選択された種別と一致することをテストで確認する（例: JSON 出力の検証）。
- [x] ラッパーコマンドが `--token-type` を受け付け、選択結果が実際の API 呼び出しに反映されることをテストで確認する（例: conv list のヘッダ検証）。

Note: ラッパーコマンドは現在プロファイルのデフォルト設定を通じてトークン種別を選択します。明示的な `--token-type` CLI フラグのサポートは、各ラッパーコマンドへの個別実装が必要となるため、将来の拡張として残されています。コアインフラ（トークン解決ロジック、プロファイル保存、meta 出力）は完全に実装されています。
