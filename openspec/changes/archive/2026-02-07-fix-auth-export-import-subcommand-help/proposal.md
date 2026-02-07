# 提案: auth export/import のサブコマンドヘルプを有効化する

## 概要
Issue #26 に対応し、`slack-rs auth export -h/--help` と `slack-rs auth import -h/--help` がエラーではなくサブコマンド固有ヘルプを表示するようにする。

## 背景
- 現在は `Unknown option: -h` / `Unknown option: --help` となり、標準的な CLI 期待に反する
- 利用者は `slack-rs auth --help` まで戻る必要があり、操作性が低い

## 目的
- `auth export` と `auth import` のローカルヘルプを直接表示できるようにする
- ヘルプ表示時の終了コードを 0 に統一する

## 非目的
- `auth` 以外のサブコマンドのヘルプ仕様変更
- export/import の機能追加や暗号化仕様変更

## 提案内容
- `auth export` / `auth import` で `-h` と `--help` を受理し、usage/options を表示する
- ヘルプ表示時は副作用なし（ファイル書き込み・復号・Keyring 更新なし）
- 回帰テストで exit code 0 と出力内容を固定する

## 期待効果
- CLI の学習コストと誤操作を削減
- スクリプト・運用手順での説明が単純化
