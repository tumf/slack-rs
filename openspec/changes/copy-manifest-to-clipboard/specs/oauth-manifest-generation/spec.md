# oauth-manifest-generation 変更仕様

## MODIFIED Requirements

### Requirement: Manifest 生成時にクリップボードへコピーする

`auth login` 実行中に Manifest YAML が生成・保存された後、その YAML を OS のクリップボードへコピーしなければならない (MUST)。
クリップボード操作が失敗した場合は警告を表示し、処理を中断してはならない (MUST NOT)。

#### Scenario: クリップボードが利用可能な場合にコピーされる
- Given `auth login` を実行して Manifest YAML を生成する
- When マニフェストがファイルに保存される
- Then 同じ YAML がクリップボードへコピーされる

#### Scenario: クリップボードが利用できない場合は警告のみで継続する
- Given クリップボード操作が失敗する環境で `auth login` を実行する
- When マニフェストを保存した後にクリップボードコピーを試行する
- Then 警告が表示され、ログイン処理は継続する
