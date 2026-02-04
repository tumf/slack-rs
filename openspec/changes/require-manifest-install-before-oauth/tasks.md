# Tasks

1. `auth login` のフローで OAuth 開始前にマニフェスト生成・保存を行う
   - 対象: `src/auth/commands.rs`
   - 完了条件: マニフェスト生成と保存が `perform_oauth_flow` 呼び出しより前に配置されていることを確認する

2. Slack App 管理ページの自動起動と案内メッセージを追加する
   - 対象: `src/auth/commands.rs`
   - 完了条件: `https://api.slack.com/apps` を開く処理が OAuth 開始前に実行されること、失敗時のフォールバック表示があることを確認する

3. ユーザー確認待ち（Enter 入力）を追加する
   - 対象: `src/auth/commands.rs`
   - 完了条件: ユーザーの Enter 入力を待つ処理が OAuth 開始前に呼ばれることを確認する

## Future Work
- マニフェストを Slack App 管理ページで実際にインストールする手順の自動化（外部サイト操作のため）
