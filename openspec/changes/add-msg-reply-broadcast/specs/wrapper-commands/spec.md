# wrapper-commands Specification

## MODIFIED Requirements
### Requirement: msg post はスレッド返信を指定できる
`msg post` は `--thread-ts` が指定された場合に `chat.postMessage` へ `thread_ts` を渡さなければならない。(MUST)
#### Scenario: thread_ts を指定してスレッド返信を送る
- Given `--thread-ts` が指定されている
- When `msg post` を実行する
- Then `chat.postMessage` に `thread_ts` が渡される

### Requirement: reply_broadcast はスレッド返信のときのみ指定できる
`msg post` は `--reply-broadcast` が指定された場合に `reply_broadcast=true` を渡さなければならない。(MUST)
`msg post` は `--thread-ts` なしで `--reply-broadcast` が指定された場合にエラーで終了しなければならない。(MUST)
#### Scenario: reply_broadcast を指定してスレッド返信を送る
- Given `--thread-ts` と `--reply-broadcast` が指定されている
- When `msg post` を実行する
- Then `chat.postMessage` に `reply_broadcast=true` が渡される
