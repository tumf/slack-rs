# 設計: 会話探索 UX

## フィルタ仕様
`conv list` に以下のフィルタを追加する。
- `--filter "name:<glob>"` : glob で名前一致
- `--filter "is_member:true|false"` : 参加有無
- `--filter "is_private:true|false"` : 公開/非公開

複数の `--filter` は AND 条件で評価する。`--types private_channel` と `--joined` は既存オプションを前提に維持し、フィルタと併用可能にする。

## インタラクティブ選択
`conv select` と `conv history --interactive` は、会話一覧を取得して対話リストを表示する。
- 表示形式: `#name (ID)`、非公開は `[private]` を付与
- キャンセル時は明示的なエラーとして扱う
- 対話 UI は抽象化し、テストではスタブを利用する
