## MODIFIED Requirements

### Requirement: トップレベルコマンド分岐の内部リファクタ後も CLI 挙動は維持される
`main` のトップレベルコマンド分岐を内部的に分割・抽出した後も、既存コマンドの引数解釈、標準出力/標準エラー出力、終了コードの挙動は後方互換でなければならない。(MUST)

#### Scenario: main 分割後も既存コマンド挙動が変わらない
- Given 既存のコマンド入力（`api call`, `auth login/status`, `conv list/history`, `msg post`, `file download`, `doctor`）を実行する
- When 内部的に `main` の分岐がハンドラ関数へ抽出された実装で実行する
- Then コマンドごとの成功/失敗判定は従来どおりである
- And 非対話エラー時の終了コード 2 を含む終了コード規約は維持される
- And 既存の JSON エンベロープおよびヘルプ出力互換性は維持される
