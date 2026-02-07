# Proposal: Broaden default conversation discovery

## 背景
`conv search` や `conv list` でチャンネル探索を行う際、既定動作が狭く（実質 public_channel 側に寄る）、
さらに 1 ページ分のみの取得に依存するため、存在するチャンネルに辿り着けないケースが発生する。

今回の会話でも、目的の `avacus-infra-merchants`（private）に到達するまでに
`types` 明示・`limit` 拡大・全件確認が必要になった。

## 目的
- 既定で「広く取得してから絞る」探索体験にする
- private channel を含む探索をデフォルト化する
- ページネーションを自動追従して、見落としを減らす

## 変更範囲
- `conv list` の既定 `types` 解決を `public_channel,private_channel` に変更
- `conv list` の既定 `limit` を探索向けに拡大（1000）
- `conversations.list` の `next_cursor` を追跡して複数ページを統合
- `conv search` / `conv select` / `conv history --interactive` の探索経路でも同じ既定動作を適用

## 非目標
- Slack API 側の検索エンドポイント追加
- `--types` 明示時の既存挙動変更
- 取得結果の表示フォーマット仕様変更

## 成功条件
- `conv list` 無指定実行で private channel が候補に含まれる
- 複数ページに跨るチャンネルが探索結果に現れる
- `conv search <pattern>` が既定設定だけで private channel を見つけられる
