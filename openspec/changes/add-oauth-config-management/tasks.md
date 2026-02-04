# Tasks

1. プロファイル設定にOAuth非機密情報（client_id/redirect_uri/scopes）を追加できるデータモデルを定義する
   - 参照: `src/profile/types.rs` の Profile 構造体
   - 完了条件: `profiles.json` へのシリアライズ/デシリアライズが可能
   - 検証: 単体テストで新フィールドを含むJSONが往復する

2. OAuth設定の解決優先順位を実装する（CLI引数 > 設定ファイル > Keyring > プロンプト）
   - 参照: `src/auth/commands.rs` の login処理
   - 完了条件: 設定ファイルに値がある場合、プロンプト無しでログイン準備ができる
   - 検証: ユニットテストで優先順位が期待通りになることを確認

3. OAuth設定の管理コマンドを追加する（set/show/delete）
   - 参照: `src/main.rs` およびCLIルーティング
   - 完了条件: `slackrs config oauth set/show/delete` が実行できる
   - 検証: コマンド出力と設定ファイル/Keyringの更新をテストで確認

4. `client_secret` を設定ファイルに保存しないことを保証する
   - 参照: `src/profile/token_store.rs`
   - 完了条件: `client_secret` はKeyringのみ保存される
   - 検証: `profiles.json` に `client_secret` が含まれないことをテストで確認

5. OAuth設定に環境変数を使わないことを明示する
   - 参照: CLIヘルプ/ドキュメント
   - 完了条件: ヘルプに環境変数の案内が存在しない
   - 検証: `slackrs --help` または該当ドキュメントの確認
