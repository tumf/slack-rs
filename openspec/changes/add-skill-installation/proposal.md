# 変更提案: `@skills/slack-rs` の埋め込み導入

## 背景
`slack-rs` にはスキル導入の標準コマンドがなく、配布元や導入方法が運用依存になっています。
この変更では、`skills/slack-rs` に配置された `@skills/slack-rs` 相当の内容を埋め込み資産として扱い、`install-skill` の既定導入をネットワーク非依存で実行できるようにします。

## 方針理由
スキルはコマンドバイナリに埋め込み、コマンドのバージョンに追従して配布します。
これにより、実行時の外部取り込みに依存せず、導入結果の再現性と配布整合性を維持します。

## 目的
- `skills/slack-rs` を埋め込み資産として扱う仕様を定義する
- `install-skill` 引数なしの既定動作を外部 GitHub 取得ではなく `self` に統一する
- スキル導入結果をロックファイルで追跡し、再現性を確保する

## スコープ
- `install-skill` の CLI 仕様（引数なし時の `self` 解決を含む）
- 埋め込み資産 (`skills/slack-rs`) とローカル (`local:<path>`) の導入経路
- スキル配置先とロックファイル更新の要件
- `commands --json` / `schema --command install-skill` への反映

## スコープ外
- 実行時に外部 GitHub から既定スキルを直接取得する運用
- GitHub/GitLab 認証フローや UI 連携
- スキルの検索、更新、アンインストール

## 成果物
- `openspec/changes/add-skill-installation/specs/skill-installation/spec.md`
- `openspec/changes/add-skill-installation/design.md`
- `openspec/changes/add-skill-installation/tasks.md`
