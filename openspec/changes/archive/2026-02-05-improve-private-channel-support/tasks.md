- [x] `conversations.list` で `types=private_channel` が指定された場合に User Token を優先する分岐を追加し、`--token-type` 指定時は優先しないことをテストで確認する。
- [x] User Token が存在しない場合のエラーメッセージにガイダンス文を追加し、文言が仕様通りであることをテストで確認する。
- [x] Bot Token で `private_channel` が空だった場合にガイダンスを表示するようにし、HTTP モックで応答ケースを再現して検証する。
- [x] `api call conversations.list` と `conv list` の両方で同じ挙動になることをテストで確認する。

## Acceptance #1 Failure Follow-up
- [x] `conv list --types private_channel` で User Token が保存されていない場合は Bot Token にフォールバックせずエラーを返すようにする（`src/cli/mod.rs` の `get_api_client_with_token_type` が User Token 不在時に警告して Bot Token を返している）。
- [x] `conv list --types private_channel` のガイダンス表示は Bot Token 使用時のみになるように判定条件を追加する（`src/cli/mod.rs` の `should_show_conv_list_guidance` が token 種別を見ていない）。

## Acceptance #2 Failure Follow-up
- [x] `src/cli/mod.rs` の `get_api_client` が `get_api_client_with_token_type(..., true)` を使うため、User Token がない環境で `search`/`users info`/`msg post` など Bot Token で動作すべきコマンドがすべて失敗する。`private_channel` を要求するケース以外は Bot Token にフォールバックできるように修正する。
