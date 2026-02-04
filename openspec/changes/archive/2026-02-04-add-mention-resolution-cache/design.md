# 設計: メンション解決とユーザーキャッシュ

## 方針
- `users.list` をページネーションで取得し、ワークスペース単位でキャッシュする
- メンション解決はローカルキャッシュから行い、未命中は原文を保持する
- キャッシュは TTL 24 時間を標準とし、手動更新を優先する

## キャッシュ構造
- 1 ファイルで複数ワークスペースのキャッシュを保持する
- 保存場所は OS 標準の設定ディレクトリ配下に固定する

```
UsersCacheFile {
  caches: {
    "T123": {
      team_id: "T123",
      updated_at: 1700000000,
      users: {
        "U123": { id, name, real_name, display_name, deleted, is_bot }
      }
    }
  }
}
```

## キャッシュ更新
- `users cache-update` で `users.list` を実行し、全ユーザーを取得する
- `limit` は 200 を標準とし、`response_metadata.next_cursor` で継続取得する
- `--force` がある場合は TTL を無視して更新する

## メンション解析
- 正規表現: `<@(U[A-Z0-9]+)(?:\|[^>]+)?>`
- 置換結果は `@display_name` をデフォルトとする
- `display_name` が空の場合は `name` を使用する
- `deleted` の場合は `@name (deleted)` を付加する

## CLI 仕様
- `users cache-update [--profile=NAME] [--all] [--force]`
- `users resolve-mentions <text> [--profile=NAME] [--format=display_name|real_name|username]`

## エラーハンドリング
- キャッシュ未作成時は警告を出し、原文を返す
- キャッシュ更新に失敗した場合はエラー終了する
