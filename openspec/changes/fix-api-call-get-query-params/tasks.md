## 1. API クライアント拡張

- [x] 1.1 `ApiClient::call` に query params を渡せる引数を追加する（Verify: `src/api/client.rs` の `call` / `execute_request` で `request.query(...)` が適用されている）
- [x] 1.2 呼び出し箇所のシグネチャ更新によりビルドエラーが解消される（Verify: `src/api/call.rs` で新しい引数を渡している）

## 2. api call の GET 送信ロジック

- [x] 2.1 GET の場合は `args.to_form()` を query として渡し、body は `None` にする（Verify: `src/api/call.rs` の `execute_api_call` で GET 時の分岐がある）
- [x] 2.2 POST の場合は既存の JSON/フォーム送信を維持する（Verify: `src/api/call.rs` の POST 分岐が既存ロジックを踏襲している）

## 3. テスト強化（httpmock）

- [x] 3.1 `test_api_call_with_get_method` で query param を検証する（Verify: `tests/api_integration_tests.rs` に `query_param("user", ...)` が追加されている）
- [x] 3.2 `conversations.replies` 相当の GET + query を検証するテストを追加する（Verify: `tests/api_integration_tests.rs` に `channel` と `ts` を query で検証する新規テストがある）
