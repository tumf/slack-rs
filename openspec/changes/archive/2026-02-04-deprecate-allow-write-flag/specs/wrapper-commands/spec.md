# wrapper-commands

## REMOVED Requirements

### Requirement: write 操作は `--allow-write` が必須
write 操作は `--allow-write` が無い場合に拒否されなければならない。(MUST)
#### Scenario: `msg post` を `--allow-write` なしで実行する
- Given write 操作を実行する
- When `--allow-write` が指定されていない
- Then エラーで終了する

## ADDED Requirements

### Requirement: write 操作は環境変数で制御される
write 操作は `SLACKCLI_ALLOW_WRITE` の値で許可/拒否を決定しなければならない。(MUST)
`SLACKCLI_ALLOW_WRITE` が未設定の場合は write 操作を許可しなければならない。(MUST)
`--allow-write` は要求されず、指定されても挙動に影響してはならない。(MUST)

#### Scenario: `SLACKCLI_ALLOW_WRITE` 未設定で `msg post` を実行する
- Given write 操作を実行する
- When `SLACKCLI_ALLOW_WRITE` が未設定
- Then write 操作が許可される

#### Scenario: `SLACKCLI_ALLOW_WRITE=false` で `msg post` を実行する
- Given write 操作を実行する
- When `SLACKCLI_ALLOW_WRITE` が `false` または `0` に設定されている
- Then エラーで終了する

#### Scenario: `--allow-write` を指定して `msg post` を実行する
- Given `SLACKCLI_ALLOW_WRITE` が未設定
- When `--allow-write` が指定されている
- Then write 操作が許可される
