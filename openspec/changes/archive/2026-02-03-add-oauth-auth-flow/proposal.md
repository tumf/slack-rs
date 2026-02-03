# 変更提案: OAuth 認証と auth コマンド

## 背景
Slack CLI はユーザトークンを取得するための OAuth 認証が必要であり、複数アカウント対応の入口として auth コマンドを提供する必要がある。

## 目的
- PKCE + localhost callback による OAuth 認証
- `auth` コマンド群の提供（login/status/list/rename/logout）
- 認証結果をプロファイル/トークン保存基盤へ連携

## 対象範囲
- OAuth 認可 URL 生成と callback 処理
- `oauth.v2.access` 交換処理
- auth コマンドの CLI ルーティング

## 対象外
- Slack API の一般呼び出し
- ラッパーコマンド

## 依存関係
- プロファイル/トークン保存基盤（establish-profile-storage）

## リスク
- ブラウザ起動失敗時の UX
- ローカルポート利用の失敗

## 成功条件
- `auth login` で OAuth 認証が完了し、プロファイルが保存される
- `auth status/list/rename/logout` が動作する
