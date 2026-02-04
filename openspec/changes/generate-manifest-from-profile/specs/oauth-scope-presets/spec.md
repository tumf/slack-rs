# oauth-scope-presets 仕様（差分）

## MODIFIED Requirements

### Requirement: スコープ入力でプリセット名を受け付ける

スコープ入力は `bot:all` と `user:all` のプリセット名を受け付けなければならない (MUST)。

また、利便性のために `all` も受け付けなければならない (MUST)。`all` は入力コンテキストに応じて次のように解釈しなければならない (MUST)。

- bot スコープ入力（例: `--bot-scopes`、または bot スコープの対話プロンプト）では `all` は `bot:all` と同義
- user スコープ入力（例: `--user-scopes`、または user スコープの対話プロンプト）では `all` は `user:all` と同義

後方互換として、旧来の単一スコープ入力（`scopes`）で `all` が指定された場合は `bot:all` と同義に扱わなければならない (MUST)。

#### Scenario: プリセット名がそれぞれ展開される
- Given `--bot-scopes` に `all` が含まれる
- And `--user-scopes` に `all` が含まれる
- When スコープを解決する
- Then bot 側は `bot:all` として展開される
- And user 側は `user:all` として展開される
