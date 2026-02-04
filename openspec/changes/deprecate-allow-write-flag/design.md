# 設計

## 変更方針
- write 操作の許可判定は環境変数 `SLACKCLI_ALLOW_WRITE` に集約する。
- `SLACKCLI_ALLOW_WRITE` 未設定の場合は許可する（default: allow）。
- `SLACKCLI_ALLOW_WRITE` が `false` または `0` の場合は write 操作を拒否する。
- `--allow-write` は CLI 仕様から削除し、指定されても挙動に影響しない。

## 互換性と移行
- 既存の `--allow-write` を利用していたユーザーは、環境変数設定に移行する。
- 既存スクリプトは `--allow-write` が無くても成功するため、互換性は維持される。

## エラーメッセージ
- write 拒否時のエラーは環境変数の利用を案内する文言に変更する。
