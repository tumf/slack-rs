## ADDED Requirements

### Requirement: Message payload includes resolved users array

When `conv history` or `thread get` returns messages, every message object MUST contain a `users` array listing all user IDs that appear in the `user` field or in `<@Uxxxx>` mentions within `text` or `blocks`. Each entry MUST include `id`, `name`, `display_name`, and `real_name` (populated from the workspace users cache when available).

#### Scenario: Poster only message

**Given**: A message with `user: "U12345678"` and no mentions in text.
**When**: The message is returned by `conv history`.
**Then**: The message contains `"users": [{"id": "U12345678", "name": "...", "display_name": "...", "real_name": "..."}]`.

#### Scenario: Message with mentions

**Given**: A message whose `text` contains `<@U87654321>` and `<@U11223344>`.
**When**: The message is returned by `thread get`.
**Then**: The `users` array contains the poster plus the two mentioned users (deduplicated, sorted by ID).

#### Scenario: Uncached user

**Given**: A user ID that does not exist in the local users cache.
**When**: The message is enriched.
**Then**: The user still appears in the `users` array with only the `id` field populated (graceful degradation, no error).

## MODIFIED Requirements

None in this change.

## REMOVED Requirements

None in this change.
