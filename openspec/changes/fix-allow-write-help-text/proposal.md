## 背景
CLI の usage には write 操作が `--allow-write` を要求する旨の記載が残っているが、現在の挙動は `SLACKCLI_ALLOW_WRITE` で制御される。仕様上も `--allow-write` は必須ではなく、ヘルプ表記との不一致が混乱を招く。

## 目的
- usage 表記を `SLACKCLI_ALLOW_WRITE` に合わせる
- `--allow-write` が必須であるような誤解を解消する

## 対象範囲
- `msg`/`react` など write 操作を含む CLI usage 表記

## 対象外
- write 制御ロジックの変更
- `--allow-write` フラグの挙動

## 依存関係
- wrapper-commands 仕様

## 成功条件
- usage 表記が `SLACKCLI_ALLOW_WRITE` を参照する
- `requires --allow-write` の表記が消える
