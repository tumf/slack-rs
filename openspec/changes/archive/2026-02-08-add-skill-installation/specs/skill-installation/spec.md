# skill-installation 仕様

## 目的
`agent-skills-rs` を利用して `install-skill` コマンドでスキルを導入できるようにする。
`@skills/slack-rs` 相当の内容は `skills/slack-rs` を埋め込み資産として扱い、既定導入をネットワーク非依存にする。
スキルはコマンドバイナリに埋め込み、コマンドのバージョンに追従して配布する。

## 方針理由
- 実行時の外部取り込みを前提にすると、導入対象が実行環境や時点で変動し再現性が低下する
- バイナリ埋め込みで配布すれば、利用者はコマンドのバージョンに対応したスキルを一貫して導入できる

## ADDED Requirements

### Requirement: install-skill は引数なし時に self を既定解決する
`slack-rs install-skill` が引数なしで実行された場合、実行時の既定ソースとして `self` を解決しなければならない。(MUST)
`self` は `skills/slack-rs` の埋め込み資産を参照しなければならない。(MUST)
引数なし実行は `self` のみを導入対象とし、外部ソース探索を行ってはならない。(MUST NOT)
インストール先は `~/.config/slack-rs/.agents/skills/<skill-name>` の正規パスでなければならない。(MUST)
`~/.config/slack-rs/.agents/.skill-lock.json` にロック情報を更新しなければならない。(MUST)

#### Scenario: 引数なしで self から導入する
- Given `install-skill` を引数なしで実行する
- When 既定ソースとして `self` が解決される
- Then `skills/slack-rs` の埋め込み資産が参照される
- And `~/.config/slack-rs/.agents/skills/<skill-name>` にスキルが配置される
- And ロックファイルが更新される

### Requirement: install-skill は self と local の source 形式を受け付ける
`install-skill <source>` は以下の source 形式を受け付けなければならない。(MUST)
- `self`
- `local:<path>`
未知のスキームはエラーにしなければならない。(MUST)

#### Scenario: local ソースでスキルを導入する
- Given `local:/path/to/skills` に `SKILL.md` が存在する
- When `slack-rs install-skill local:/path/to/skills` を実行する
- Then 指定パスからスキルを検出してインストールする

#### Scenario: self 指定で埋め込みスキルを導入する
- Given `slack-rs install-skill self` を実行する
- When 埋め込みスキルが検出される
- Then 埋め込みスキルをインストールする

#### Scenario: 未知スキームはエラーになる
- Given `slack-rs install-skill foo:bar` を実行する
- When ソース解釈に失敗する
- Then 許容されるスキーム一覧を含むエラーが表示される

### Requirement: インストールは symlink を優先し失敗時に copy へフォールバックする
`install-skill` は symlink を優先し、作成に失敗した場合は copy にフォールバックしなければならない。(MUST)

#### Scenario: symlink 失敗時に copy へ切り替える
- Given symlink を作成できない環境である
- When `install-skill` を実行する
- Then copy モードでスキルが配置される

### Requirement: install-skill は JSON 出力を返す
`install-skill` の出力は JSON で、`ok`/`type`/`schemaVersion` を含まなければならない。(MUST)
インストール結果として `skills` 配列に `name`/`path`/`source_type` を含めなければならない。(MUST)

#### Scenario: JSON 出力でインストール結果を返す
- Given `install-skill` を実行する
- When インストールが成功する
- Then JSON で結果が出力される
- And `skills` 配列にインストール情報が含まれる

### Requirement: イントロスペクションに install-skill が含まれる
`commands --json` は `install-skill` を含むコマンド一覧を返さなければならない。(MUST)
`schema --command install-skill --output json-schema` は出力スキーマを返さなければならない。(MUST)

#### Scenario: commands に install-skill が含まれる
- Given `slack-rs commands --json` を実行する
- Then `install-skill` がコマンド一覧に含まれる

#### Scenario: install-skill の schema が取得できる
- Given `slack-rs schema --command install-skill --output json-schema` を実行する
- Then JSON Schema が出力される
