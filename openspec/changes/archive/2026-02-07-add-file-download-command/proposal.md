# 変更提案: file download コマンドの追加

## 背景
- 現在の `slack-rs` は `file upload` を提供している一方、Slack 添付ファイルを CLI 単体でダウンロードする手段がない。
- 利用者は `files.info` で URL を取得したうえで、外部ツール（例: `curl`）に Bearer トークンを手動設定して取得する必要がある。
- この運用では URL 種別の取り違え（permalink vs `url_private_download`）や認証ヘッダ不足により、期待するバイナリではなく HTML を取得してしまう失敗が発生しやすい。

## 目的
- `slack-rs file download` を追加し、既存プロフィール/トークン解決を使った認証付きダウンロードを一貫した UX で提供する。
- `<file_id>` 指定時に `files.info` から `url_private_download`（fallback: `url_private`）を自動解決する。
- `--out` 指定、`--out -`（stdout ストリーム）、非 2xx と HTML 応答時の明確なエラーを提供する。

## 変更スコープ
- CLI ルーティングへの `file download` 追加（`main` の `file` サブコマンド配下）。
- `run_file_download` 相当の引数解釈と既存 `--profile` / `--token-type` 連携。
- `commands::file` にダウンロード実装を追加し、`files.info` 呼び出し + 認証付き GET を実行。
- デフォルト出力先の安全なファイル名決定、`--out -` の stdout ストリーム対応。
- 失敗時のメッセージ改善（特に `Content-Type: text/html` へのヒント）。
- モックベースのテスト（HTTP モック）で外部資格情報なしに検証可能にする。

## 非目的
- Slack API の仕様変更や新 API 追加。
- 実ファイルの進捗表示、再開ダウンロード、並列分割ダウンロード。
- `file upload` の挙動変更。

## 前提と制約
- 本変更は read 操作であり、`SLACKCLI_ALLOW_WRITE` の制御対象外とする。
- 外部依存（実 Slack ワークスペース/実トークン）に依存しないよう、提案とタスクはモック検証を優先する。

## 想定ユーザー価値
- 手動でトークンや `curl` を扱わずに、安全かつ再現可能なファイル取得が可能になる。
- 誤 URL/認証不備時に HTML 応答を早期検出でき、原因切り分けが容易になる。
