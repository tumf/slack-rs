# wrapper-commands Specification (Delta)

## MODIFIED Requirements

### Requirement: Conv list retrieves conversation list
`conv list` MUST accept `--include-private` and `--all` and reflect them in `types` resolution. (MUST)
When both `--types` and `--include-private`/`--all` are specified simultaneously, an error MUST be returned. (MUST)

#### Scenario: Specify `--include-private`
- Given `--types` is not specified
- When executing `conv list --include-private`
- Then `public_channel,private_channel` is passed to `types`

#### Scenario: Specify `--all`
- Given `--types` is not specified
- When executing `conv list --all`
- Then `public_channel,private_channel,im,mpim` is passed to `types`

#### Scenario: Use `--types` and `--all` together
- Given `--types=public_channel` is specified
- When executing `conv list --types=public_channel --all`
- Then an error is returned for simultaneous use of `--types` and `--all`

### Requirement: Wrapper commands show guidance for known Slack error codes
When `channel_not_found` is returned, cause candidates and next actions MUST be displayed to stderr. (MUST)

#### Scenario: `channel_not_found` guidance is displayed
- Given Slack API returns `ok=false` and `error=channel_not_found`
- When executing `conv history C123`
- Then stderr displays possibilities of "private and not joined", "token type", and "profile" with verification methods

## ADDED Requirements

### Requirement: Conv search matches are user-friendly by default
`conv search` MUST use case-insensitive substring matching by default. (MUST)
When the pattern contains `*`, glob matching MUST be used. (MUST)

#### Scenario: Search with case-insensitive substring matching
- When executing `conv search Gen`
- Then `general` is treated as a matching candidate

#### Scenario: Use glob matching when `*` is present
- When executing `conv search "gen*"`
- Then channel names starting with `gen` are matched
