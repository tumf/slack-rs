- [x] `wrapper-commands` の spec delta を更新し、`file download` が 3xx リダイレクトを追従する要求を追加する（確認方法: `openspec/changes/2026-02-07-harden-file-download-ux/specs/wrapper-commands/spec.md` に該当 Requirement/Scenario が存在すること）。
- [x] `wrapper-commands` の spec delta を更新し、`text/html` 応答時にヒントに加えて短い本文スニペットを含める失敗要件を追加する（確認方法: 同 spec delta に Scenario と期待出力条件が明記されていること）。
- [x] モックベースの検証タスクを定義し、外部 Slack 資格情報なしで確認可能にする（確認方法: tasks 内に 302 モックと HTML モックでの検証観点が明示されていること）。

## モックベースの検証タスク

### 302 リダイレクトの検証
- [x] モック HTTP サーバーを用意し、初回 GET が 302 (Location ヘッダー付き) を返し、追従先が 200 OK でファイル本体を返す動作を再現する
- [x] `file download` がリダイレクト先まで追従し、最終的にファイルを保存できることを確認する
- [x] 確認方法: モックサーバーでの統合テストが成功し、保存されたファイル内容が期待通りであること（tests/file_download_integration.rs にて実装済み）

### text/html 応答時の診断情報表示の検証
- [x] モック HTTP サーバーを用意し、`Content-Type: text/html` と短い HTML 本文を返す
- [x] `file download` がエラー終了し、エラーメッセージに以下の要素が含まれることを確認する:
  - URL 種別不一致または認証問題を示すヒントメッセージ
  - 本文先頭の短いスニペット（安全に切り詰められたもの）
- [x] 確認方法: モックサーバーでの統合テストが成功し、エラーメッセージに期待されるメッセージとスニペットが含まれること（tests/file_download_integration.rs にて実装済み）
- [x] 変更提案を `npx @fission-ai/openspec@latest validate 2026-02-07-harden-file-download-ux --strict` で検証し、成功させる（確認方法: コマンド終了コード 0 と "Change ... is valid" の出力）。

## Acceptance #1 Failure Follow-up

- [x] `file_download` で `Content-Type: text/html` 判定を HTTP ステータス判定より前（または非 2xx 時も本文取得する分岐）に移し、非 2xx の HTML 応答でもヒントと本文スニペットを必ず返す。
- [x] 3xx リダイレクト時に最終到達先まで `Authorization: Bearer` を維持して取得できることを実装で保証し、同一ホスト/別ホスト両方のモック統合テストで検証する。
- [x] `モックベースの検証タスク` セクションの各項目を完了済み (`- [x]`) として明示するか、未実施なら `Future Work` へ移動してチェックボックスを外す。

## Acceptance #2 Failure Follow-up

- [x] 3xx リダイレクトの別ホスト遷移（最初のモックサーバーから別のモックサーバーへ `Location`）を追加し、最終到達先 GET でも `Authorization: Bearer` が送信されることを統合テストで実証する。
- [x] 上記の別ホスト統合テストが実装・成功するまで、`Acceptance #1 Failure Follow-up` の「同一ホスト/別ホスト両方のモック統合テストで検証する」を完了扱いにしない（`- [x]` の見直し、または実装後に再度 `- [x]` 化）。
