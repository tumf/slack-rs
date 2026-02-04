# 設計: プロファイル別OAuth設定管理

## 方針
- OAuth設定はプロファイル単位で管理し、単一の環境変数設定は廃止する。
- 秘密情報はKeyringに保存し、設定ファイルには保存しない。
- ログイン時の解決優先順位は「CLI引数 > 設定ファイル > Keyring > プロンプト」とする。
  - `client_id`, `redirect_uri`, `scopes` は設定ファイルから取得可能
  - `client_secret` はKeyringから取得し、無ければプロンプト

## データモデル
- `profiles.json` 内の各プロファイルにOAuth設定の非機密情報を追加する。

例:
```json
{
  "version": 1,
  "profiles": {
    "default": {
      "team_id": "T123",
      "user_id": "U456",
      "client_id": "111.222",
      "redirect_uri": "http://127.0.0.1:3000/callback",
      "scopes": ["chat:write", "users:read"]
    }
  }
}
```

## CLIコマンド
- OAuth設定管理のために以下を追加する。

例:
```
slackrs config oauth set --profile work --client-id ... --redirect-uri ... --scopes "chat:write,users:read"
slackrs config oauth show --profile work
slackrs config oauth delete --profile work
```

- `set` は `client_secret` の入力を求め、Keyringに保存する（設定ファイルには保存しない）。
- `show` は `client_secret` を表示しない。

## 互換性
- 既存の `profiles.json` にOAuth設定が無くても読み込み可能。
- 既存のログインフローは、設定がない場合に従来通り対話入力へフォールバックする。

## セキュリティ
- `client_secret` はKeyringに保存し、設定ファイルには書き込まない。
- `show` コマンドは秘密情報を出力しない。
