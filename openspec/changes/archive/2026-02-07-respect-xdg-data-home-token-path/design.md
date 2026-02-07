# 設計: XDG_DATA_HOME を考慮した token path 解決

## 設計方針
- 変更範囲は `FileTokenStore` のデフォルトパス解決と、その解決結果を参照する表示経路に限定する
- 既存の明示オーバーライド `SLACK_RS_TOKENS_PATH` を最優先で維持する
- 既存利用者への影響を最小化するため、`XDG_DATA_HOME` 未設定時は現行の `~/.local/share/slack-rs/tokens.json` にフォールバックする

## パス解決ルール
- 優先順位
  - 1. `SLACK_RS_TOKENS_PATH`
  - 2. `XDG_DATA_HOME/slack-rs/tokens.json`
  - 3. `~/.local/share/slack-rs/tokens.json`
- `XDG_DATA_HOME` が空文字または無効値の場合は 2 をスキップして 3 を使用する

## 影響範囲
- `FileTokenStore::default_path()`
- `auth status` の `Token Store: file (...)` 表示（`default_path()` の返り値を表示しているため間接影響）

## テスト方針（mock-first）
- 実ホームディレクトリや実認証情報を使わず、環境変数と一時ディレクトリでパス解決を検証する
- 外部サービスや認証は不要なため、すべてローカルテストで検証可能
- 少なくとも以下を回帰テスト化する
  - `XDG_DATA_HOME` 設定時に `$XDG_DATA_HOME/slack-rs/tokens.json` が解決される
  - `SLACK_RS_TOKENS_PATH` が `XDG_DATA_HOME` より優先される
  - `XDG_DATA_HOME` 未設定時は従来パスにフォールバックする

## トレードオフ
- `XDG_DATA_HOME` を参照する分、環境依存条件が増える
- ただし、XDG 準拠・テスト容易性・運用予測可能性の利点が上回る
