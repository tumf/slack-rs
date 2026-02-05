# 設計メモ: CLI ハンドラー分離

## 方針
- `src/main.rs` のコマンド分岐を保持しつつ、具体処理は専用モジュールに移動する
- 既存の公開関数や外部インターフェースは変更しない

## 変更概要
- `src/cli/handlers.rs`（または同等の新規モジュール）に以下を移動
  - `run_auth_login`
  - `run_api_call`
  - `handle_export_command`
  - `handle_import_command`
- `main.rs` は引数解析とハンドラー呼び出しに限定

## 代替案
- `src/main.rs` のまま小分割する案
  - ファイル分割による責務分離が弱く、将来の拡張時に再び肥大化する

## 影響範囲
- `src/main.rs`
- `src/cli/mod.rs`（必要に応じて re-export を追加）

## 互換性
- CLI の振る舞い・引数・出力は変更しない
