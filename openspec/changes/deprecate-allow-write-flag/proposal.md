# 変更提案: --allow-write 廃止と環境変数による write 制御

## 背景
現在のラッパーコマンドでは write 操作に `--allow-write` が必須となっている。運用上は環境変数による一括制御のほうが扱いやすく、実行時のフラグ要求を廃止したい。

## 目的
- write 操作の許可/拒否を環境変数でのみ制御する
- デフォルトは許可（allow）とする
- `--allow-write` を廃止し、CLI から不要な指示を除外する

## 対象範囲
- `msg` / `react` など write 操作を伴うラッパーコマンド
- write ガードの判定ロジックとエラーメッセージ
- CLI 使用方法と関連ドキュメントの更新

## 対象外
- 汎用 `api call`
- OAuth 認証フロー
- トークン/プロフィール管理

## 依存関係
- wrapper-commands 仕様

## 成功条件
- `--allow-write` なしで write 操作が実行可能（環境変数未設定時）
- `SLACKCLI_ALLOW_WRITE=false` または `0` で write 操作が拒否される
-  usage/README から `--allow-write` の記載が削除される
