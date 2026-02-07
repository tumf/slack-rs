# 設計: auth import --dry-run

## 設計方針
- 既存 import の評価ロジックを再利用し、永続化フェーズのみ抑止する
- 実適用と dry-run で判定ロジックを共有し、結果不一致を避ける

## 動作要件
- `--dry-run` 時は `profiles.json` と token store に書き込まない
- 競合判定後の予定 action を出力する
- `--force` 併用時は「上書き予定」として報告する

## テスト方針（mock-first）
- 一時ディレクトリの設定ファイルと token store を使い、実行前後でファイル差分ゼロを検証する
- `--dry-run --json` で予定 action が JSON 取得できることを確認する

## トレードオフ
- オプション分岐が増える
- ただし誤更新防止の価値が高い
