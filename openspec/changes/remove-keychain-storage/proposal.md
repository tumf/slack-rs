# macOS Keychain Removal Proposal

## 背景
macOS の Keychain を使用すると、ビルドごとに許可が無効化される等の問題が発生し、CLI の実用性が低下している。

## 目的
macOS Keychain と Keyring 依存を完全に削除し、トークンストレージを FileTokenStore に一本化する。

## スコープ
- Keychain/Keyring 実装・依存の削除
- `SLACKRS_TOKEN_STORE` 環境変数の削除
- デフォルトのトークンストレージを FileTokenStore に固定
- 仕様・ドキュメント・CLI 出力から Keychain/Keyring/環境変数の痕跡を削除

## 非スコープ
- 互換性や移行手順の提供
- 既存 Keychain データの移行

## 影響
- `SLACKRS_TOKEN_STORE` は廃止され、参照されない
- FileTokenStore が常に使用される

## 受け入れ基準
- リポジトリ内に Keychain/Keyring への参照が残らない
- `SLACKRS_TOKEN_STORE` への参照が残らない
- 既定のトークンストレージが FileTokenStore である
