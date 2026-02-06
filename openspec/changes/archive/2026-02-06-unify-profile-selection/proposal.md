# Proposal: Unify profile selection across CLI

## 背景
現在の CLI では `--profile` と `SLACK_PROFILE` の扱いがコマンド間で不一致であり、
特に `api call` は `--profile` を読まず、wrapper コマンドは `SLACK_PROFILE` を考慮していません。
このため、ユーザーが意図しない profile が使われる、または profile 指定方法が不明確になる問題が発生しています。

## 目的
- 全コマンドで profile 解決の優先順位を統一する
- `--profile` をグローバルフラグとして一貫して利用できるようにする
- `--profile <name>` と `--profile=<name>` の両形式をどの位置でも受け付ける

## 範囲
- CLI ルーティング前の引数正規化
- profile 解決ヘルパーの共通化
- `api call` と wrapper コマンド全体への適用

## 非目的
- トークン種別選択ロジックの変更（#17 とは別）
- 出力形式や write 確認の仕様変更

## 成功条件
- `--profile` が前置/後置どちらでも有効
- `--profile` > `SLACK_PROFILE` > `default` の優先順位が明確
- `api call` を含む全コマンドで同一の profile 解決となる
