# 提案: auth login で既定トークン種別を永続化する

## 背景
Issue #17 で、`auth status` の「Default Token Type: User」表示と `api call` の実際のトークン選択が一致しないケースが報告された。調査の結果、`auth login` によるプロファイル作成時に `default_token_type` が常に `None` として保存され、`auth status` は推測表示を行うため、表示と実挙動の乖離が起きる。

## 目的
- `auth login` 完了時に、`default_token_type` が未設定であれば推測値を保存し、表示と実挙動を一致させる。
- 既存の優先順位（`--token-type` > `profile.default_token_type` > 推測）を変更しない。

## 変更概要
- `auth login` がプロファイルを保存する際、`default_token_type` が未設定の場合に限り、取得できたトークンに基づいて既定値を決めて保存する。
  - User トークンが取得できた場合は `user` を既定とする。
  - User トークンがない場合は `bot` を既定とする。
- 既に `default_token_type` が設定されているプロファイルは上書きしない。

## 非目的
- `api call` のトークン解決優先順位の変更。
- `SLACK_TOKEN` 環境変数の優先ルール変更。
- `auth status` の表示形式の変更。

## 影響範囲
- `auth login` のプロファイル保存ロジック
- `auth login` に関するユニットテスト

## 受け入れ基準
- `auth login` で User トークンが取得できた場合、`default_token_type=user` がプロファイルに保存される。
- `auth login` で User トークンが取得できない場合、`default_token_type=bot` が保存される。
- 既に `default_token_type` が設定済みのプロファイルは上書きされない。
