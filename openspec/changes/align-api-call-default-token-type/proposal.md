# 提案: api call の既定トークン種別を一貫させる

## 背景
Issue #17 で、`auth status` の「Default Token Type: User」表示と `api call` 実際のトークン選択が一致しないケースが報告された。プロフィールに Bot/User 両方のトークンがある状態で、`default_token_type` が未設定のとき、`auth status` は User を推測表示するが、`api call` は Bot を既定として選択してしまう。

## 目的
- `api call` の未指定時の既定トークン種別を、ユーザートークンの存在に基づいて推測し、`auth status` の表示と挙動を一致させる。
- 既存の `--token-type` と `profile.default_token_type` の優先順位は維持する。

## 変更概要
- `api call` のトークン解決で、`--token-type` 未指定かつ `profile.default_token_type` 未設定の場合、トークンストアに User token が存在すれば User を既定として選択し、なければ Bot を選択する。
- 既存の明示指定 (`--token-type` または `profile.default_token_type`) がある場合は、これまで通りその指定を優先する。

## 非目的
- `auth status` の表示形式の変更や出力項目の追加は行わない。
- 既存の `SLACK_TOKEN` 優先ルールやエラーメッセージの文言変更は行わない。

## 影響範囲
- `api call` のトークン解決ロジック
- `api call` のトークン選択に関するテスト

## リスクと対策
- 既定の選択が Bot から User に変わることで、一部の環境でトークン選択が変化する可能性がある。
  - 明示指定 (`--token-type` / `default_token_type`) がある場合は従来通りのため、必要な場合は既定設定を利用して固定できる。

## 受け入れ基準
- `default_token_type` 未設定かつ User token が存在する場合、`api call` は User token を使用する。
- `default_token_type` 未設定かつ User token が存在しない場合、`api call` は Bot token を使用する。
- `--token-type` または `default_token_type` が設定されている場合、その指定が優先される。
