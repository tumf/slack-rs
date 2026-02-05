# 設計メモ: conv コマンド実装のモジュール分割

## 方針
- 既存の公開 API と CLI 挙動を維持する
- 責務単位でモジュールを分割し、`mod.rs` で再公開する

## 変更概要
- `src/commands/conv/` 配下に分割
  - `filter.rs`: フィルタ/マッチング
  - `sort.rs`: ソート処理
  - `format.rs`: 出力整形
  - `api.rs`: API 呼び出し
  - `select.rs`: 対話選択
- `src/commands/conv/mod.rs` で既存の関数名を re-export

## 代替案
- 1ファイルのまま関数順序のみ整理
  - 責務の分離が弱く、変更容易性の改善が限定的

## 影響範囲
- `src/commands/conv.rs`（分割後は `src/commands/conv/mod.rs`）
- `src/commands/mod.rs`
- `src/cli/mod.rs`

## 互換性
- 既存の関数名と CLI 挙動は変更しない
