1. `skills/slack-rs` を埋め込み資産として扱う仕様に更新する
   検証: `design.md` と `specs/skill-installation/spec.md` に `skills/slack-rs` を `self` の参照元として明記されている

2. `install-skill` 引数なしの既定動作を `self` に統一する
   検証: `specs/skill-installation/spec.md` の要件・シナリオで、引数なし時に外部 GitHub ではなく `self` が解決される記述になっている

3. `install-skill` の受け付けソースを `self` と `local:<path>` に整理する
   検証: `specs/skill-installation/spec.md` の source 形式要件が `self` と `local:<path>` のみになっている

4. 外部取り込み前提を削除し、配布方針の理由を明記する
   検証: `proposal.md` / `design.md` / `specs/skill-installation/spec.md` に「スキルはコマンドバイナリに埋め込み、コマンドのバージョンに追従して配布する」理由が記載されている

5. 出力仕様とイントロスペクション仕様を整合させる
   検証: `specs/skill-installation/spec.md` に `ok`/`type`/`schemaVersion`、`skills[].name/path/source_type`、`commands --json`、`schema --command install-skill` の要件が記載されている

6. 変更提案ドキュメント間の整合性を確認する
   検証: `proposal.md` / `design.md` / `tasks.md` / `specs/skill-installation/spec.md` の記述が、引数なし `self` 既定・埋め込み配布方針で一致している
