# Design

## 方針
- トークンストレージを FileTokenStore のみに統一する
- Keychain/Keyring の実装・依存・文言はすべて削除する
- `SLACKRS_TOKEN_STORE` 環境変数を削除する
- 互換性や移行は考慮しない

## 変更点
- `SLACKRS_TOKEN_STORE` は廃止し、参照をすべて削除する
- トークンストレージは常に FileTokenStore を使用する
- Keyring 関連のエラー文言やガイダンスを削除する
- デモ/ヘルプ/README/レシピ等から Keychain/Keyring の記述を削除する

## リスク
- 既存の Keychain データは利用されない

## 判断
互換性は不要という要件に従い、移行やフォールバックは実装しない
