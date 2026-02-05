# 提案: CLI のスペース区切りオプション対応

## 背景
Issue #16 で `conv list` の `--filter` / `--sort` / `--format` が動作しないと報告されました。調査の結果、CLI が `--option=value` 形式のみを受理しており、スペース区切り形式 (`--option value`) を受け付けていないことが原因と判断しました。

## 問題
- ユーザーは `--filter "is_private:true"` のようなスペース区切り形式を使用している
- 現在の実装は `--filter=is_private:true` のみ認識するため、フィルタ・ソート・フォーマットが適用されない
- `--token-type` のみ両形式をサポートしており、CLI 全体として一貫性がない

## 目的
- 値を取る CLI オプションで `--option=value` と `--option value` の両方を受け付ける
- 既存の `--option=value` 形式の動作は維持する（後方互換）
- `conv list` の `--filter` / `--sort` / `--format` がスペース区切りでも動作するようにする

## 非目的
- 新しいオプションの追加
- 出力フォーマットや API 呼び出し仕様の変更
- `--token-type` 以外の既存挙動の破壊的変更（例: エラー文言の変更）

## 変更概要
- `src/cli/mod.rs` のオプション取得ロジックを拡張し、スペース区切りの値を拾えるようにする
- `--filter` のような複数指定オプションでも混在形式を受理する
- ヘルプ表示に「スペース区切り形式も受理」する旨を追記する（必要最小限）

## 影響範囲
- 影響コマンド: `conv list` / `conv search` / `conv select` / `conv history` / `search` など
- 外部 API 仕様への影響なし
- テストの追加・更新（ユニットテスト）

## 成果物
- 仕様差分（wrapper-commands）
- 実装タスク一覧（tasks.md）
- 実装方針の簡易設計（design.md）
