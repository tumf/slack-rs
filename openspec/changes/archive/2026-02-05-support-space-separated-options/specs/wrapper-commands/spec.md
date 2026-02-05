# wrapper-commands

## ADDED Requirements

### Requirement: `conv list` accepts space-separated options
Value-taking options for `conv list` (`--filter`/`--format`/`--sort`/`--sort-dir`/`--types`/`--limit`/`--profile`) MUST accept both `--option=value` and `--option value` formats equivalently. (MUST)

#### Scenario: Execute `conv list` with space-separated options
- Given executing `conv list --filter "is_private:true" --sort name --sort-dir desc --format table`
- When retrieving conversation list and applying filter and sort
- Then only channels matching `is_private:true` are output in descending order in `table` format
