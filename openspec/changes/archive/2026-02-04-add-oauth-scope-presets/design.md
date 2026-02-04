# 設計メモ: OAuthスコープのプリセット

## 方針
- プリセット名 `all` を入力値として受け付ける
- `all` は内部の固定リストへ展開し、入力に含まれる他スコープと結合して重複除去する
- 展開後のスコープは安定した順序で保持する

## `all` のスコープ一覧
以下は包括的な用途を想定したセットとする（管理者/Enterprise限定は除外）。

```
chat:write,users:read,channels:read,channels:history,channels:write,groups:read,groups:history,groups:write,im:read,im:history,im:write,mpim:read,mpim:history,mpim:write,files:read,files:write,usergroups:read,usergroups:write,team:read,emoji:read,reactions:read,reactions:write,pins:read,pins:write,stars:read,stars:write,reminders:read,reminders:write,search:read,dnd:read,dnd:write,users.profile:read,users.profile:write,conversations.connect:read,conversations.connect:write
```

## 影響範囲
- 対話入力の解析（`auth` コマンド群）
- `config oauth set --scopes` の解析
- 表示/保存されるスコープの正規化
