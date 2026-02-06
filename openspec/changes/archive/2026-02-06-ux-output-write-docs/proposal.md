# Proposal: Output ergonomics, write safety, debug, and recipes

## 背景
出力の envelope は有用だが、`jq` での処理では raw が欲しい場面が多い。
また write 操作は誤実行のリスクがあり、確認プロンプトの一貫性が不足している。
さらに、`--debug/--trace` と help/recipes の強化は日常利用の摩擦を減らす。

## 目的
- `SLACKRS_OUTPUT=raw|envelope` による既定出力の切替を提供する
- write 操作に統一された確認プロンプトを追加する
- `--debug/--trace` で実行コンテキストを可視化する
- 主要な例を help と `docs/recipes.md` に追加する

## 範囲
- `api call` と wrapper コマンドの出力既定
- write 操作の確認フロー
- debug/trace フラグの追加
- recipes ドキュメントの新設

## 非目的
- Slack API 仕様の変更
- トークン種別選択の修正

## 成功条件
- `SLACKRS_OUTPUT=raw` で `--raw` なしでも raw 出力
- write 操作は TTY で確認が入り、`--yes` でスキップできる
- `--debug/--trace` で解決済み情報を stderr に表示
- `docs/recipes.md` が追加され、help に例がある
