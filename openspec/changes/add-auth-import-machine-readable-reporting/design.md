# 設計: auth import の結果レポート構造化

## 設計方針
- import 本体ロジックは維持し、結果収集と出力レイヤを追加する
- `text` と `json` を同一の内部結果モデルから生成し、整合性を保つ

## 出力モデル
- 全体サマリ: `updated` / `skipped` / `overwritten` 件数
- 明細: profile ごとに `action`（updated/skipped/overwritten）と理由

## テスト方針（mock-first）
- 実 credential は使わず、fixture 化した import データで分岐を再現する
- `--yes`、`--force`、`--json` の組み合わせを統合テストで検証する

## トレードオフ
- 出力仕様の維持コストは増える
- ただし運用可観測性と自動化適合性が向上する
