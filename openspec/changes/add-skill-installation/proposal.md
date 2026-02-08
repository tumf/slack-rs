# 変更提案: Agent Skills の導入と install-skill コマンド

## 背景
`slack-rs` にエージェントスキルの導入機能がなく、スキルの配布・インストールを CLI から行えません。
`agent-skills-rs` を導入することで、埋め込みスキルと外部ソースの両方を統一的に扱えるようになります。

## 目的
- `agent-skills-rs` を依存関係として導入する
- `install-skill` コマンドでスキルをインストールできるようにする
- インストール結果をロックファイルで追跡し、再現性を確保する

## スコープ
- `install-skill` の CLI 追加とエントリポイントの配線
- スキルの格納先・ロックファイル運用の定義
- GitHub/ローカル/埋め込みスキルのインストール経路
- CLI イントロスペクションへのコマンド登録

## スコープ外
- 実際の GitHub/GitLab 認証フローや UI 連携
- スキル検索・一覧表示 UI
- 既存スキルの更新・アンインストール

## 成果物
- `openspec/changes/add-skill-installation/specs/skill-installation/spec.md`
- `openspec/changes/add-skill-installation/tasks.md`
- `openspec/changes/add-skill-installation/design.md`
