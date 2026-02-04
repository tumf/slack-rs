# 変更提案: msg post のスレッド返信ブロードキャスト対応

## 背景
現状の `msg post` はチャンネルへの通常投稿のみを想定しており、スレッド返信や reply_broadcast を指定できない。

## 目的
- `msg post` でスレッド返信を指定できるようにする
- 必要に応じて reply_broadcast を指定できるようにする

## 対象範囲
- `msg post` の CLI オプション追加（`--thread-ts`, `--reply-broadcast`）
- `chat.postMessage` へのパラメータ送信
- usage 表示と関連ヘルプの更新

## 対象外
- `msg update` / `msg delete` へのスレッド機能追加
- 既存の `api call` コマンド
- OAuth 認証やトークン管理

## 依存関係
- wrapper-commands 仕様

## 成功条件
- `msg post --thread-ts=<ts>` で `thread_ts` が送信される
- `msg post --thread-ts=<ts> --reply-broadcast` で `reply_broadcast=true` が送信される
- `--reply-broadcast` 単独指定時にエラーで終了する
