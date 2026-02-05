# wrapper-commands 変更仕様（会話探索）

## ADDED Requirements
### Requirement: conv list にフィルタを追加する
`conv list` は `--filter` で `name:<glob>` と `is_member:true|false` と `is_private:true|false` を受け付け、AND 条件で絞り込むこと。 (MUST)
#### Scenario: 名前と参加状態で絞り込む
- Given `conv list --filter "name:ark*" --filter "is_member:true"` を実行する
- When 会話一覧を取得する
- Then 条件に一致するチャンネルのみが出力される

### Requirement: conv select を提供する
`conv select` は会話一覧を対話表示し、選択したチャンネル ID を返すこと。 (MUST)
#### Scenario: 対話選択でチャンネル ID を取得する
- Given `conv select` を実行する
- When 一覧からチャンネルを選択する
- Then 選択したチャンネル ID が出力される

### Requirement: conv history にインタラクティブ選択を追加する
`conv history --interactive` は会話一覧からチャンネルを選び、その ID で履歴取得を行うこと。 (MUST)
#### Scenario: チャンネル選択後に履歴取得する
- Given `conv history --interactive` を実行する
- When 対話でチャンネルを選択する
- Then 選択したチャンネルの履歴が取得される
