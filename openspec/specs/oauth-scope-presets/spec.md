# oauth-scope-presets Specification

## Purpose
TBD - created by archiving change add-oauth-scope-presets. Update Purpose after archive.
## Requirements
### Requirement: OAuthスコープ入力でプリセット名を受け付ける

対話入力および `--scopes` 引数は、プリセット名 `all` を入力値として受け付けなければならない (MUST)。

#### Scenario: `all` を指定した場合に展開される
- Given `auth login` のスコープ入力または `config oauth set --scopes` に `all` が含まれる
- When スコープを解決する
- Then 事前定義された包括的スコープ一覧に展開される

### Requirement: `all` と個別スコープが混在した場合に正規化する

`all` と個別スコープが混在した場合、展開後に重複が除去され、安定した順序で保存されなければならない (MUST)。

#### Scenario: `all` と追加スコープの混在
- Given 入力が `all,users:read` のように混在している
- When スコープを解決する
- Then 展開後の一覧から重複が除去され、安定順序で保持される

