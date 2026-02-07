- [x] `wrapper-commands` の spec delta を更新し、`file download` が 3xx リダイレクトを追従する要求を追加する（確認方法: `openspec/changes/2026-02-07-harden-file-download-ux/specs/wrapper-commands/spec.md` に該当 Requirement/Scenario が存在すること）。
- [x] `wrapper-commands` の spec delta を更新し、`text/html` 応答時にヒントに加えて短い本文スニペットを含める失敗要件を追加する（確認方法: 同 spec delta に Scenario と期待出力条件が明記されていること）。
- [x] モックベースの検証タスクを定義し、外部 Slack 資格情報なしで確認可能にする（確認方法: tasks 内に 302 モックと HTML モックでの検証観点が明示されていること）。

## モックベースの検証タスク

### 302 リダイレクトの検証
- モック HTTP サーバーを用意し、初回 GET が 302 (Location ヘッダー付き) を返し、追従先が 200 OK でファイル本体を返す動作を再現する
- `file download` がリダイレクト先まで追従し、最終的にファイルを保存できることを確認する
- 確認方法: モックサーバーでの統合テストが成功し、保存されたファイル内容が期待通りであること

### text/html 応答時の診断情報表示の検証
- モック HTTP サーバーを用意し、`Content-Type: text/html` と短い HTML 本文を返す
- `file download` がエラー終了し、stderr に以下の要素が含まれることを確認する:
  - URL 種別不一致または認証問題を示すヒントメッセージ
  - 本文先頭の短いスニペット（安全に切り詰められたもの）
- 確認方法: モックサーバーでの統合テストが成功し、stderr に期待されるメッセージとスニペットが含まれること
- [x] 変更提案を `npx @fission-ai/openspec@latest validate 2026-02-07-harden-file-download-ux --strict` で検証し、成功させる（確認方法: コマンド終了コード 0 と "Change ... is valid" の出力）。
