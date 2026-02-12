## 設計方針

### 1. フォーマット互換性
- `ExportProfile` に `user_token` をオプションフィールドとして追加する
- 既存の `token` フィールドは維持し、従来データはそのまま読み込める
- `format_version` は変更せず、未知フィールド許容の仕様を利用して前方互換も確保する

### 2. エクスポート対象の判定
- bot token または user token のどちらかが存在すればエクスポート対象とする
- どちらも存在しない場合のみ「トークン未保存」としてスキップ/エラー扱いにする

### 3. インポート時の保存
- `token` は従来どおり bot token として `team_id:user_id` に保存
- `user_token` があれば `team_id:user_id:user` に保存
- `user_token` が欠落していても import 失敗にはしない

### 4. テスト戦略
- 既存の export/import テストに user token の保存/復元確認を追加
- 片方のトークンのみ存在するケースをユニットテストで検証
