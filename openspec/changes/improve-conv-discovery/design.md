# Design: Conversation helpers

## `conv list` の types 解決
- `--types` が指定されていない場合にのみショートハンドを解釈する。
- `--include-private` は `public_channel` に `private_channel` を追加する。
- `--all` は `public_channel,private_channel,im,mpim` を指定する。
- `--types` と `--include-private`/`--all` の同時指定は曖昧なためエラーにする。

## `conv search` のマッチング
- パターンに `*` を含む場合は既存の glob マッチを維持する。
- `*` が無い場合は大小無視の部分一致をデフォルトとする。

## `channel_not_found` ガイダンス
- `channel_not_found` を既知エラーとして扱い、
  private/未参加/トークン種別/誤った profile の可能性を示す短いヒントを出す。
