# 変更提案: api call の GET で key=value を query に送る

## 背景
`slack-rs api call <method> ... --get` で `key=value` を指定しても、GET リクエストに query が付与されず、Slack 側で必須フィールド不足になる不具合がある。
Issue #28 の再現例では `conversations.replies` へ `channel` と `ts` を渡しているが、Slack では `missing required field` と判定される。

## 目的
- `--get` 指定時に `key=value` を query params として送信する
- 既存の POST/JSON/フォーム送信の挙動を保持する
- 既存のテスト基盤（httpmock）で検証できること

## 非目的
- Slack 実環境への疎通検証
- API 呼び出し仕様全体の再設計

## 変更概要
- GET 時に `ApiCallArgs.params` を query として付与できるように API クライアント呼び出しを拡張
- `execute_api_call` で GET の場合は query を付与し、body は送らない
- 既存の統合テストを query 検証に強化し、`conversations.replies` の再現に近いケースを追加

## 影響範囲
- `src/api/call.rs`
- `src/api/client.rs`
- `tests/api_integration_tests.rs`

## リスク
- `ApiClient::call` の引数変更による影響
  - 対策: 呼び出し箇所は `execute_api_call` のみであるため、合わせて更新する

## 受け入れ基準
- `--get` 指定時、`key=value` が query params として送信される
- `--json` と `--get` を併用した場合でも、GET では query 優先で body は送られない
- 既存の POST + JSON/フォーム送信の挙動が維持される
- 追加/更新テストが通る（httpmock で検証）
