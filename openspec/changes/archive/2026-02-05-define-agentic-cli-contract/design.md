# 設計ノート: 非対話モードの導入

## 目的
`slack-rs` をエージェント実行や CI のようなヘッドレス環境で安全に実行できるようにし、入力待ちによる停止を防ぐ。

## 設計方針
- **非対話を明示化**: `--non-interactive` をグローバルに導入し、対話が必要な場合は即時エラーにする。
- **自動判定**: TTY が存在しない環境では `--non-interactive` を暗黙有効にする。
- **復旧可能なエラー**: エラーには「どの入力が不足しているか」「次に何を指定すべきか」を含める。

## 対象の対話ポイント
- 破壊操作の確認プロンプト（例: `msg delete`, `react remove`）
- `auth login` の OAuth 設定入力プロンプト（client_id / client_secret / redirect_uri / scopes）

## エラー設計
- `--non-interactive` 時にプロンプトが必要な場合は、
  - **エラー種別**: `non_interactive` を示すコードを付与
  - **ガイダンス**: 不足している引数や再実行例を提示
- 実行は即終了し、stdin を読まない

## TTY 判定
- `std::io::IsTerminal` を利用し、stdin が terminal でない場合は non-interactive を強制
- 明示的に `--interactive` を導入する案もあるが、本提案では `--non-interactive` のみとする

## 互換性
- 通常の対話実行は従来どおり動作
- `--non-interactive` が無い場合でも、TTY が無い環境では明示エラーに切り替わるため挙動は変わる

## 未対応（今後の検討）
- JSON 出力のエラースキーマ統一
- `--output` など出力フォーマットの一元化
