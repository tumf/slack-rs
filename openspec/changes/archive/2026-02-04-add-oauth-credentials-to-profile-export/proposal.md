## 背景
現在のprofile export/importはアクセストークンのみを対象としており、OAuthクライアント情報（client_id/client_secret）が含まれていません。
複数ワークスペース運用や別マシン移行時に、OAuthクレデンシャルを別途手動で用意する必要があり、運用負荷と漏えいリスクが高まります。

## 目的
profile export/importにOAuthクレデンシャルを含め、暗号化されたバックアップ/移行が可能になるようにします。

## スコープ
- exportペイロードに `client_id` と `client_secret` を含める（存在する場合のみ）
- import時に `client_id` をprofiles設定へ、`client_secret` をKeyringへ復元する
- 既存のexportファイルと既存profilesとの後方互換を維持する

## 非スコープ
- OAuthクライアント情報の取得/入力フローの変更
- export/importの暗号方式やファイルフォーマットの大規模変更
- 既存のtoken保存方式の変更

## 依存関係
- `add-per-profile-oauth-credentials` で導入される `client_id` と `client_secret` の保存方式

## 成功条件
- exportにOAuthクレデンシャルが含まれる（存在する場合）
- importでOAuthクレデンシャルが復元される（存在する場合）
- 既存のexportファイルが読み込める
- 既存profiles.jsonが読み込める

## 影響範囲
- `src/auth/format.rs` のペイロード定義
- `src/auth/export_import.rs` のエクスポート/インポート処理
- Keyringに保存するOAuthクライアントシークレットの取得/保存
