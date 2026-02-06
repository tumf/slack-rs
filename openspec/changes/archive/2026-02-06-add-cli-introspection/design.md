# デザイン: CLI自己記述化

## 方針
最小限の実装で Introspectable を満たすため、CLI ルーティングに対応した静的な「コマンド定義テーブル」を導入し、以下の 3 種の出力を生成する。

1. `commands --json`: コマンド一覧とフラグの機械可読一覧
2. `--help --json`: 個別コマンドの構造化ヘルプ
3. `schema --command <cmd> --output json-schema`: 出力の JSON Schema

既存の JSON エンベロープは互換性を保ったまま `schemaVersion`/`type`/`ok` を追加する。

## コマンド定義テーブル
CLI のルーティングに合わせ、以下の属性を持つデータ構造を用意する。

- `name`: コマンド名（例: `conv list`）
- `description`: 説明
- `usage`: 使用例
- `flags`: フラグ配列（`name`, `type`, `required`, `description`, `default`）
- `examples`: 代表的な実行例
- `exit_codes`: CLI の終了コード一覧
- `error_codes`: CLI で扱うエラーコード一覧（既存のガイダンス対象も含む）

このテーブルから `commands --json` と `--help --json` を生成する。

## 出力形式

### `commands --json`
```json
{
  "schemaVersion": 1,
  "type": "commands.list",
  "ok": true,
  "commands": [
    {
      "name": "conv list",
      "description": "List conversations",
      "flags": [
        {"name": "--limit", "type": "integer", "required": false}
      ]
    }
  ]
}
```

### `--help --json`
```json
{
  "schemaVersion": 1,
  "type": "help",
  "ok": true,
  "command": "msg post",
  "usage": "slack-rs msg post <channel> <text> [flags]",
  "flags": [
    {"name": "--thread-ts", "type": "string", "required": false}
  ],
  "examples": [
    {"description": "Post message", "command": "slack-rs msg post C123 hello"}
  ],
  "exitCodes": [
    {"code": 0, "description": "Success"},
    {"code": 2, "description": "Invalid arguments"}
  ]
}
```

### `schema --command <cmd> --output json-schema`
- 目的は **CLI の出力仕様を機械可読にすること**であり、入力値の検証は最小化する
- `schemaVersion`/`type`/`ok`/`response`/`meta` を定義に含める

## エンベロープ拡張
既存の `response`/`meta` は維持し、以下を追加する。

- `schemaVersion`: 現行は `1`
- `type`: コマンド識別子（例: `conv.list`, `auth.status`）
- `ok`: CLI 処理の成否（Slack API の `ok` と区別）

`--raw` を指定した場合は既存通り Slack API レスポンスのみを返す。

## テスト方針
- `commands --json` の出力が JSON としてパース可能であることをユニットテストで検証
- `--help --json` の必須フィールドを検証
- `schema` 出力が JSON Schema として最低限の構造を持つことを検証
- 既存の `--raw` 出力に影響がないことを既存テストで担保
