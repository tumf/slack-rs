---
title: SlackCLI 認証情報の暗号化 Export/Import 追加
status: draft
change-id: add-auth-export-import
---

# 変更提案: 認証情報 Export/Import（Keyring 互換）

## Why
SlackCLI の profile（ワークスペースごとの OAuth 認証情報）を端末間で移行・バックアップできるようにしつつ、通常運用は OS の secure store（Keychain/Secret Service 等）に保持する。gog の keyring export/import に相当する暗号化エクスポートを提供し、漏洩リスクを最小化する。

## What Changes
- `slackcli auth export` / `slackcli auth import` を追加する
- export は必ず暗号化し、平文出力を禁止する
- passphrase は env または prompt で取得する
- export/import フォーマットは `format_version` を持ち、将来拡張に備える
- i18n（ja/en）で警告・プロンプトを出し分ける

## スコープ
- CLI コマンドと引数の追加
- Keyring への保存/読み出し
- 暗号化ファイルの読み書きと検証
- i18n メッセージ追加
- テストの追加（暗号化・ファイル権限・i18n・CLI）

## 非スコープ
- 実際の Slack OAuth フローの取得・更新
- 既存の profile 管理方式の変更（keyring への保存以外）
- 実機のクロスプラットフォーム検証（Future Work）

## 依存関係 / 影響
- 既存の profile ストレージ（`establish-profile-storage`）が前提
- OAuth 取得（`add-oauth-auth-flow`）の結果を保存できる必要がある

## リスクと対策
- **漏洩リスク**: export は暗号化必須 + 危険操作の確認 + 0600 権限を強制
- **誤操作**: `--yes` なしでは export 失敗、import は競合検知
- **互換性**: `format_version` を固定し、unknown field を無視

## Future Work
- 実機での Keyring 互換性検証（macOS/Windows/Linux）
- 追加の暗号方式（KMS, age, gpg など）
