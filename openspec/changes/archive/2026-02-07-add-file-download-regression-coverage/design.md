# 設計: file download 入力モード別の回帰固定

## 設計方針
- 既存の `file download` 実行経路を変えず、仕様とテストで回帰点を明確化する
- 外部依存を避けるため、Slack API は HTTP モックで検証する

## 追加する検証軸
- `file download <file_id>` で `files.info` へ正しい形式の `file` パラメータが送信される
- `file download --url <private_url>` で認証ヘッダ付き直接取得が行われる

## テスト方針（mock-first）
- 画像・動画のフィクスチャ応答を用意し、実ファイルや実トークンに依存しない
- 失敗時は `invalid_arguments` を検知できるアサーションを入れる

## トレードオフ
- テストケースが増え実行時間はわずかに増える
- ただし再発抑止の利得が上回る
