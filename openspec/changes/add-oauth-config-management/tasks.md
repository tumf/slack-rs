# Tasks

- [x] 1. プロファイル設定にOAuth非機密情報（client_id/redirect_uri/scopes）を追加できるデータモデルを定義する
   - 参照: `src/profile/types.rs` の Profile 構造体
   - 完了条件: `profiles.json` へのシリアライズ/デシリアライズが可能
   - 検証: 単体テストで新フィールドを含むJSONが往復する

- [x] 2. OAuth設定の解決優先順位を実装する（CLI引数 > 設定ファイル > Keyring > プロンプト）
   - 参照: `src/auth/commands.rs` の login処理
   - 完了条件: 設定ファイルに値がある場合、プロンプト無しでログイン準備ができる
   - 検証: ユニットテストで優先順位が期待通りになることを確認

- [x] 3. OAuth設定の管理コマンドを追加する（set/show/delete）
   - 参照: `src/main.rs` およびCLIルーティング
   - 完了条件: `slackrs config oauth set/show/delete` が実行できる
   - 検証: コマンド出力と設定ファイル/Keyringの更新をテストで確認

- [x] 4. `client_secret` を設定ファイルに保存しないことを保証する
   - 参照: `src/profile/token_store.rs`
   - 完了条件: `client_secret` はKeyringのみ保存される
   - 検証: `profiles.json` に `client_secret` が含まれないことをテストで確認

- [x] 5. OAuth設定に環境変数を使わないことを明示する
   - 参照: CLIヘルプ/ドキュメント
   - 完了条件: ヘルプに環境変数の案内が存在しない
   - 検証: `slackrs --help` または該当ドキュメントの確認

## Acceptance #1 Failure Follow-up

- [x] OAuthログイン時に `redirect_uri` と `scopes` が未設定の場合、対話入力を行わず既定値で補完しているため、仕様の「不足時のみ対話入力」に反する。`src/auth/commands.rs` の `login_with_credentials` を要修正
   - 完了: `prompt_for_redirect_uri` と `prompt_for_scopes` 関数を追加し、設定ファイルに値がない場合は対話入力を行うように変更
   - 検証: 設定値がない場合、デフォルト値を提示しつつユーザーに確認を求める実装を追加
- [x] `config oauth set` で未ログインのプロファイルを作ると `team_id`/`user_id` が `PLACEHOLDER` になり、ログイン後の `save_profile_and_credentials` が `set_or_update` で `DuplicateName` となる。`src/commands/config.rs` と `src/profile/types.rs` の整合性を修正
   - 完了: `set_or_update` に PLACEHOLDER 値の特別処理を追加
   - PLACEHOLDER から実際の値への更新を許可
   - 実際の値から PLACEHOLDER への降格を防止
   - PLACEHOLDER 値は他のプロファイルとの重複チェックから除外
   - 検証: 3つの新しいテストケースを追加して動作確認
