## 目的
`conv list` の既存機能を活かしつつ、検索用途を分かりやすくする。

## 設計方針
- **API は増やさない**: `conv search` は `conversations.list` の結果を `name` フィルタで絞り込むだけにする。
- **既存オプションを再利用**: `--types`/`--limit`/`--format`/`--sort`/`--sort-dir` を `conv search` でも利用できるようにする。
- **対話選択を共通化**: `--select` は `conv select` の選択ロジックを再利用する。
- **ヘルプを明確化**: フィルタのキー一覧とフォーマット・ソートの有効値をヘルプに明記する。

## 具体的な挙動
- `conv search <pattern>` は `name:<pattern>` のフィルタを内部的に追加して `conv list` と同様の出力を行う。
- `conv search --select` はフィルタ後のリストから 1 件選択してチャンネル ID を出力する。
- `conv list --help` には `--filter name|is_member|is_private`、`--format json|jsonl|table|tsv`、`--sort name|created|num_members`、`--sort-dir asc|desc` の説明が含まれる。

## 代替案
- `conv search` を新設せずに、ヘルプ改善のみで解決する案もあるが、初心者向けの導線が弱いため採用しない。
