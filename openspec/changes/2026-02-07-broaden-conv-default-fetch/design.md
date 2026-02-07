# Design: conv 系探索の既定拡張

## 方針
- 既定値は「探索しやすさ」を優先し、`public + private` を標準にする。
- 既存の明示指定（`--types`、`--include-private`、`--all`）は優先し、後方互換を保つ。
- 1 回の API 呼び出し結果に依存せず、`next_cursor` を辿って論理的に全件集合を構築する。

## 実装アプローチ
### 1) 型解決の既定変更
- `run_conv_list` / `run_conv_search` / `run_conv_select` / `run_conv_history --interactive` の会話一覧取得経路で、
  `types` 未指定時は `public_channel,private_channel` を使う。
- `--types` 指定時はその値をそのまま使い、`--include-private` / `--all` との排他ルールは維持する。

### 2) 既定 limit とページネーション
- `limit` 未指定時は 1000 を送る。
- `conversations.list` 呼び出しをページネーション対応にし、`response_metadata.next_cursor` が空になるまで取得する。
- 各ページの `channels` を連結し、最終レスポンスとして扱う。

### 3) 既存パイプラインとの整合
- フィルタ（`--filter`）・ソート（`--sort`）・出力（`--format`）は「全ページ統合後」に適用する。
- `--raw` は既存ルール（JSON 時のみ有効）を維持する。

## トレードオフ
- API 呼び出し回数は増えるが、探索失敗率を下げられる。
- 既定で private を含めるため、表示件数は増えるが、必要なら `--types=public_channel` で従来相当へ絞れる。

## テスト方針
- Slack 実環境に依存しないよう、既存テスト基盤でページネーション応答をモック/fixture 化する。
- 既定値解決（types/limit）と、ページ連結後のフィルタ・検索動作をユニット/統合テストで検証する。
