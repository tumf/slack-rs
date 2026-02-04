# 設計メモ: 外部アップロード方式

## 目的
`files.upload` に依存せず、Slack が推奨する外部アップロード方式でファイル添付を行う。

## 方針
- 3 ステップのフローを明示的に実装する。
  1. `files.getUploadURLExternal` で `upload_url` と `file_id` を取得
  2. `upload_url` にファイルの生バイトを POST（`Content-Type: application/octet-stream`）
  3. `files.completeUploadExternal` で共有先/コメント/タイトルを指定
- Step2 は Slack API ではないため、既存の `ApiClient::call()` を使わず、専用の `reqwest::Client` で送信する。
- 認証/プロファイル取得は既存の `cli::get_api_client` と同じ流れに合わせる。

## 影響範囲
- CLI ルーティング（`file upload` の追加）
- wrapper-commands への新規コマンド追加
- テストは HTTP モックで完結させる
