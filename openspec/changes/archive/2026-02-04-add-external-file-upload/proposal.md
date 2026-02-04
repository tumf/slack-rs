# 変更提案: 外部アップロード方式のファイル添付を追加

## 背景
`files.upload` は非推奨となっており、Slack は `files.getUploadURLExternal` + `files.completeUploadExternal` の新方式を推奨している。現状の CLI にはファイル添付コマンドがなく、`api call` でも multipart 送信ができないため、実用的なファイルアップロード手段が不足している。

## 目的
- 新方式のみでファイル添付を行える CLI コマンドを追加する
- 既存の認証・トークン管理・API クライアントの流れに沿って実装する

## 対象範囲
- `file upload` コマンドの追加（`--channel`, `--channels`, `--title`, `--comment`, `--profile`）
- 3 ステップの外部アップロードフローの実装
- usage / help 表示の更新
- モックサーバを使ったアップロードフローのテスト追加

## 対象外
- `files.upload` 互換（旧方式）の実装
- OAuth や profile 管理の仕様変更
- GUI や進捗表示、複数ファイル同時アップロード

## 依存関係
- wrapper-commands 仕様
- files:write スコープを持つトークン（実行時の前提）

## 成功条件
- `file upload <path> --allow-write` で 3 ステップの外部アップロードが実行される
- `--channel` / `--channels` / `--comment` / `--title` が `files.completeUploadExternal` に反映される
- 旧 `files.upload` を呼び出さない
