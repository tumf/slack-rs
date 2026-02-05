## MODIFIED Requirements
### Requirement: client_secret を安全に取得できる
`config oauth set` は以下の入力経路で `client_secret` を取得できなければならない。(MUST)
1) `--client-secret-env <VAR>` で指定された環境変数
2) `SLACKRS_CLIENT_SECRET`
3) `--client-secret-file <PATH>`
4) `--client-secret <SECRET>`（`--yes` による明示同意が必要）
5) 対話入力（上記がない場合）

`--client-secret` を `--yes` なしで指定した場合は安全上の理由で拒否し、利用可能な代替手段を案内しなければならない。(MUST)

#### Scenario: 環境変数から client_secret を取得する
- Given `SLACKRS_CLIENT_SECRET` が設定されている
- When `config oauth set <profile> --client-id ... --redirect-uri ... --scopes ...` を実行する
- Then 対話入力を行わずに `client_secret` を取得する
- And `client_secret` は token store backend に保存される

#### Scenario: `--client-secret` には `--yes` が必要
- Given `--client-secret` が指定されている
- And `--yes` が指定されていない
- When `config oauth set` を実行する
- Then コマンドは失敗する
- And 代替手段（環境変数/ファイル/対話入力）が案内される
