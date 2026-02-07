## 変更概要
`file download` 実行時に `files.info` へ渡す `file_id` を JSON ボディで送ってしまい、Slack API から `invalid_arguments` が返る不具合を修正する。

## 背景
- `files.info` は `application/x-www-form-urlencoded`（またはクエリ）形式のパラメータを期待する。
- 現在の `file download` 実装では `file_id` を JSON ボディとして送っており、引数解釈に失敗する。

## スコープ
- `file download` 内の `files.info` 呼び出しで、`file_id` を JSON ボディではなくフォーム/クエリとして送るように修正する。
- 既存の `files.info` 成功レスポンス処理とダウンロード処理の流れは維持する。

## スコープ外
- `file upload` や他のラッパーコマンドの送信方式変更。
- エラーメッセージ文面や出力フォーマットの全面見直し。

## 受け入れ基準
- `file download --file-id <FILE_ID>` で `files.info` が `invalid_arguments` を返さず、`file_id` が正しく受理される。
- パラメータ送信形式に関する回帰テスト（モック/スタブ）で JSON ボディ送信が再発しないことを確認できる。
