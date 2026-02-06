# 提案: CLIの自己記述化（Introspectable）を追加

## 背景
現状の slack-rs は人間向けの `--help` 出力やコマンド一覧のみで、エージェントが CLI を機械的に理解する手段が不足している。特に Agentic CLI Design の原則7（Introspectable）に必要な「コマンド発見」「構造化ヘルプ」「スキーマ出力」が未実装である。

## 目的
- CLI が自身のコマンド構造・引数・出力仕様を JSON で自己記述できるようにする
- 既存のラッパーコマンド出力に `schemaVersion`/`type`/`ok` を追加し、機械可読性を強化する

## スコープ
- `commands --json` によるコマンド一覧の JSON 出力
- `schema --command <cmd> --output json-schema` による出力スキーマの JSON Schema 出力
- `--help --json` による構造化ヘルプの出力
- 統一エンベロープに `schemaVersion`/`type`/`ok` を追加（`--raw` は除外）

## 非スコープ
- Slack API の仕様変更
- OAuth フローや認証設定の変更
- 既存出力の破壊的変更（`--raw` の挙動は維持）

## 影響範囲
- `src/cli` と `src/commands` の出力整形
- 既存の JSON エンベロープ定義
- 統合テスト・ユニットテスト

## リスクと対応
- 既存スクリプトのパースが `schemaVersion` 追加で影響する可能性
  - 対策: `response`/`meta` の構造は維持し、追加フィールドは上位互換とする
- コマンド一覧のメンテナンスコスト
  - 対策: CLI ルーティングと同一の定義テーブルから生成する
