## 変更概要
Issue #9 に対応し、`conv search` コマンドを追加し、`conv list` のフィルタ/ソート/フォーマットの利用方法をヘルプに明記する。

## 背景
- 既に `conv list` のフィルタ/ソート/フォーマットは実装されているが、ヘルプに十分な説明がなく discoverable ではない。
- 「チャンネル検索」に対しては `conv list --filter name:<glob>` が有効だが、専用コマンドがなく初心者には分かりにくい。

## スコープ
- `conv search <pattern>` を追加し、`conv list` の結果を `name` フィルタで絞り込む。
- `conv search` は `--select` をサポートし、対話的にチャンネル ID を返せるようにする。
- `conv list` の `--filter`/`--format`/`--sort`/`--sort-dir` の説明をヘルプに追加する。

## スコープ外
- Slack API 側の検索エンドポイント（サーバーサイド検索）の導入。
- 既存の `conv list` のフィルタ仕様変更。

## 既知のリスク
- `conv search` はクライアント側のフィルタのため、取得件数 (`--limit`) によって検索結果が変動する。
- 追加のヘルプ項目により `--help` 出力が長くなる。

## 受け入れ基準
- `conv search <pattern>` が `name:<pattern>` フィルタとして動作する。
- `conv search --select` が選択したチャンネル ID を出力する。
- `conv list --help` に `--filter`/`--format`/`--sort`/`--sort-dir` の説明が含まれる。
