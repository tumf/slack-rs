# 設計: `conv list` のフォーマット/ソート

## 追加オプション
### `--format <format>`
- 許容値: `json` | `jsonl` | `table` | `tsv`
- デフォルト: `json`
- 意味:
  - `json`: 既存と同様に JSON を出力する（エンベロープ/`--raw` など既存の方針に従う）
  - `jsonl`: 1 行 1 チャンネルの JSON を出力する（スクリプト向け）
  - `table`: 人間向けに列を揃えて表示する（端末幅に依存）
  - `tsv`: 1 行 1 チャンネルのタブ区切りを出力する（スクリプト向け）

### `--sort <key>`
- 許容値: `name` | `created` | `num_members`
- デフォルト: 未指定（API 返却順を維持）
- 意味:
  - `name`: `channel.name` をキーにソート（大小比較は文字列）
  - `created`: `channel.created` をキーにソート（数値、欠落時は 0）
  - `num_members`: `channel.num_members` をキーにソート（数値、欠落時は 0）

### `--sort-dir <dir>`
- 許容値: `asc` | `desc`
- デフォルト: `asc`

## 既存の `--raw` との関係
- `--raw` は出力フォーマットが JSON のとき（デフォルト/`--format json`）のみ有効/意味がある
- `--format jsonl`/`table`/`tsv` と `--raw` は互換性がないため、併用時はエラーにする

## 適用順序
1. Slack API から会話一覧を取得する（既存の `conversations.list` 呼び出し）
2. `--filter` が指定されている場合は既存仕様どおりに AND 条件で絞り込む
3. `--sort` が指定されている場合は、(2) の結果に対して安定ソートを適用する
4. `--format` に応じて出力を生成する

## 出力フィールド（`table`/`tsv`）
`table` と `tsv` は最低限以下の列を提供する。

- `id`
- `name`
- `is_private`
- `is_member`
- `num_members`

### 欠落値の扱い
- `table`/`tsv` の `num_members` は、値が欠落または `null` の場合は空欄を出力する（`0` は出力しない）。
  - 目的: `0`（実際に 0）と `unknown`（不明）を区別して意味を保つ
- 一方で、`--sort created` / `--sort num_members` のソート比較では、欠落値は 0 として扱う（既存の方針どおり）。

## エラー設計
- `--format`/`--sort`/`--sort-dir` に未知の値が与えられた場合は、許容値を含むエラーメッセージを表示して終了する
- `--format` が `jsonl`/`table`/`tsv` の場合に `--raw` が指定されていたら、互換性がないことを示すエラーを表示して終了する（例: ``--raw`` は ``--format json`` のときのみ利用可能）
- Slack API 側の失敗（`ok=false`）の取り扱いは既存のエラー方針に従う（本変更では追加の変換を行わない）
