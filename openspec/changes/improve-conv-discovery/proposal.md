# Proposal: Improve conversation discovery helpers

## 背景
`conv list` と `conv search` は日常的に使われる一方、
private チャンネルの扱いや検索のマッチ精度でユーザーの期待とズレが生じています。

## 目的
- private チャンネルの列挙を簡単にする
- `conv search` をより直感的な検索挙動にする
- `channel_not_found` の原因推定と対処方法を提示する

## 範囲
- `conv list` のフラグ追加（`--include-private`, `--all`）
- `conv search` のデフォルトマッチング改善
- `channel_not_found` ガイダンスの追加

## 非目的
- 出力形式や profile 解決の変更
- Slack API 仕様そのものの変更

## 成功条件
- `conv list --include-private` / `--all` が期待通りの types を解決する
- `conv search` が大文字小文字無視 + 部分一致をデフォルトとする
- `channel_not_found` 時に原因候補と対処が stderr に表示される
