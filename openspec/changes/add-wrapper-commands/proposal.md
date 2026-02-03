# 変更提案: ラッパーコマンドと write 安全装置

## 背景
日常運用では `search` / `conv` / `users` / `msg` / `react` のような高頻度操作を簡易に実行できる必要がある。さらに write 操作には事故防止のための安全装置が必須である。

## 目的
- 読み取りラッパー（search/conv/users）
- 書き込みラッパー（msg/react）
- `--allow-write` と `--yes` による安全装置

## 対象範囲
- `search`, `conv`, `users`, `msg`, `react` コマンド
- write 操作のガード
- 破壊操作の確認

## 対象外
- 汎用 `api call`
- OAuth 認証

## 依存関係
- プロファイル/トークン保存基盤
- 汎用 API 呼び出し

## 成功条件
- ラッパーコマンドが動作し、必要な API を呼び出す
- write 操作が `--allow-write` なしでは拒否される
- 破壊操作は `--yes` なしだと確認が入る
