# 設計メモ: file download コマンド

## 概要
`file download` は 2 つの入力モードを提供する。
- `file download <file_id>`: `files.info` を呼び出してダウンロード URL を解決してから取得する。
- `file download --url <url_private_or_download>`: 指定 URL を直接取得する。

どちらも既存のプロフィール/トークン解決（`--profile`, `--token-type`, `SLACK_TOKEN` 優先）を利用し、認証付き GET を行う。

## エントリポイント設計
- `main.rs`
  - `file` サブコマンド配下に `download` を追加。
- `src/cli/mod.rs`
  - `run_file_download(args, non_interactive)` を追加し、`--url` / `--out` / `--profile` / `--token-type` を解釈。
  - `print_file_usage()` と introspection 定義を更新。
- `src/commands/file.rs`
  - `file_download(...)` 実装を追加。

## データフロー
- `<file_id>` モード
  - `files.info(file=<file_id>)` を認証付きで実行。
  - `url_private_download` を優先、なければ `url_private` を使用。
  - URL 未取得ならエラー。
- `--url` モード
  - 指定 URL をそのまま認証付き GET。
- 出力
  - `--out -`: stdout にバイナリのみを書き込む。
  - `--out <path>`: 指定パスへ保存（ディレクトリ指定時は安全化したファイル名を連結）。
  - `--out` 省略: カレントへ安全化したファイル名で保存。

## エラーハンドリング
- ダウンロード応答が非 2xx の場合は失敗終了（非ゼロ）とする。
- `Content-Type` が `text/html` の場合は失敗として扱い、
  - 誤 URL（permalink 利用）
  - 認証ヘッダ不足/トークン権限不足
  の可能性を示すヒントを stderr に出す。
- `--out -` 時は stdout 汚染を避けるため、診断情報は stderr のみに出す。

## ファイル名決定ポリシー
- `<file_id>` モード: `files.info.file.name` を第一候補。
- 取得不能時: `file-<file_id>` などのフォールバック名を使用。
- 不正文字は置換して安全化し、空文字を回避する。
- 既存ファイル衝突時は上書きせず、サフィックス付与で回避する。

## テスト戦略（mock-first）
- `wiremock` / `httpmock` で `files.info` とダウンロード URL をモックし、以下を自動検証する。
  - `url_private_download` 優先 + `url_private` fallback。
  - `--url` 直接取得。
  - `text/html` 応答時の失敗。
  - 非 2xx 応答時の失敗。
  - `SLACKCLI_ALLOW_WRITE=false` 下でも read 操作として実行可能。

これにより、実 Slack 資格情報なしで仕様適合性を確認できる。
