# wrapper-commands 変更仕様（private_channel ガイダンス）

## ADDED Requirements
### Requirement: private_channel 取得失敗時にガイダンスを出す
`conv list` で `types=private_channel` が指定された場合、取得結果が空であり Bot Token を使用しているときはガイダンスを表示すること。 (MUST)
#### Scenario: Bot Token でプライベートチャンネルが空
- Given Bot Token を使用して `conv list --types private_channel` を実行する
- When 取得結果が空である
- Then User Token を使う/ボットを招待する旨のガイダンスが表示される

### Requirement: User Token が存在しない場合の明示エラー
User Token が存在しない状態で `private_channel` を要求した場合は、User Token が必要である旨のエラーを返すこと。 (MUST)
#### Scenario: User Token 不在
- Given User Token が保存されていない
- When `conv list --types private_channel` を実行する
- Then User Token が必要である旨のエラーが返る
