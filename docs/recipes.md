# Slack CLI Recipes

This document provides practical examples for common tasks using the Slack CLI.

## Table of Contents

- [Profile Management](#profile-management)
- [Output Format Control](#output-format-control)
- [Conversations and Threads](#conversations-and-threads)
- [Error Handling](#error-handling)

## Profile Management

### Switching Between Profiles

Use the `SLACK_PROFILE` environment variable to select a profile:

```bash
# Use default profile
slack-rs api call auth.test

# Use a specific profile
SLACK_PROFILE=work slack-rs api call auth.test

# Use a different profile for a session
export SLACK_PROFILE=personal
slack-rs conv list
slack-rs msg post C123456 "Hello from personal workspace"
```

### List All Profiles

```bash
# Show all configured profiles
slack-rs auth list

# Show detailed status for a profile
slack-rs auth status work
```

### Setting Default Token Type

```bash
# Set default token type for a profile
slack-rs config set work --token-type user

# Now all commands will use user token by default
slack-rs api call users.info user=U123456
```

## Output Format Control

### Raw vs Envelope Output

The CLI provides two output formats:

1. **Envelope format** (default): Includes metadata (profile, team_id, user_id, method)
2. **Raw format**: Returns only the Slack API response

```bash
# Default envelope output
slack-rs api call users.info user=U123456
# Output: {"response": {...}, "meta": {...}}

# Raw output with --raw flag
slack-rs api call users.info user=U123456 --raw
# Output: {"ok": true, "user": {...}}

# Raw output with environment variable (affects all commands)
export SLACKRS_OUTPUT=raw
slack-rs api call users.info user=U123456
# Output: {"ok": true, "user": {...}}

# Switch back to envelope format
export SLACKRS_OUTPUT=envelope
slack-rs api call users.info user=U123456
# Output: {"response": {...}, "meta": {...}}
```

### Using jq with Raw Output

```bash
# Extract specific fields with jq
slack-rs api call users.info user=U123456 --raw | jq '.user.name'

# Use SLACKRS_OUTPUT for cleaner scripts
export SLACKRS_OUTPUT=raw
slack-rs api call users.info user=U123456 | jq '.user.name'

# List channels and extract names
slack-rs api call conversations.list | jq '.channels[].name'
```

## Conversations and Threads

### List Conversations

```bash
# List all conversations (envelope format)
slack-rs conv list

# List with raw output
slack-rs conv list --raw

# Filter by type
slack-rs conv list --filter type:public_channel

# Interactive selection
slack-rs conv select
```

### Get Conversation History

```bash
# Get recent messages from a channel
slack-rs conv history C123456

# Get with specific limit
slack-rs conv history C123456 --limit=50

# Interactive mode with filters
slack-rs conv history --interactive --filter type:public_channel
```

### Post Messages and Threads

```bash
# Post a simple message
slack-rs msg post C123456 "Hello, team!"

# Post a threaded reply
slack-rs msg post C123456 "Reply text" --thread-ts=1234567890.123456

# Post a threaded reply visible in channel
slack-rs msg post C123456 "Important update" \
  --thread-ts=1234567890.123456 \
  --reply-broadcast
```

### Get Thread Replies

```bash
# Get all replies in a thread
slack-rs api call conversations.replies \
  channel=C123456 \
  ts=1234567890.123456 \
  --raw | jq '.messages'
```

## Error Handling

### Debug Mode

Use `--debug` or `--trace` to troubleshoot issues:

```bash
# Show debug information
slack-rs api call users.info user=U123456 --debug
# Output to stderr:
# DEBUG: Profile: default
# DEBUG: Token store: file storage/file
# DEBUG: Token type: bot
# DEBUG: API method: users.info
# DEBUG: Endpoint: https://slack.com/api/users.info

# Verbose trace mode
slack-rs api call users.info user=U123456 --trace
# Shows additional trace-level information
```

### Common Errors

#### Missing Token

```bash
# Error: No token found
slack-rs api call auth.test

# Solution: Login first
slack-rs auth login

# Or set token via environment
export SLACK_TOKEN=xoxb-your-token
slack-rs api call auth.test
```

#### Token Type Mismatch

```bash
# Error: not_allowed_token_type
slack-rs api call conversations.list --token-type=user

# Solution: Use bot token
slack-rs api call conversations.list --token-type=bot

# Or set as default
slack-rs config set default --token-type bot
```

#### Write Protection

```bash
# Error: Write operation not allowed
SLACKCLI_ALLOW_WRITE=false slack-rs msg post C123456 "Test"

# Solution: Enable write operations
SLACKCLI_ALLOW_WRITE=true slack-rs msg post C123456 "Test"
```

#### Non-interactive Mode

```bash
# Error: Operation requires confirmation in non-interactive mode
echo | slack-rs msg delete C123456 1234567890.123456

# Solution: Use --yes flag
echo | slack-rs msg delete C123456 1234567890.123456 --yes

# Or use explicit --non-interactive
slack-rs msg delete C123456 1234567890.123456 --yes --non-interactive
```

### Error Guidance

The CLI automatically provides guidance for common errors:

```bash
# Example: channel_not_found error
slack-rs api call chat.postMessage channel=INVALID text=Hello
# Output includes:
# Error: channel_not_found
# Cause: Channel ID is invalid or bot is not a member
# Resolution: Verify channel ID and add bot to channel

# Use --debug to see additional context
slack-rs api call chat.postMessage channel=INVALID text=Hello --debug
# Shows:
# DEBUG: Slack error code: channel_not_found
```

### Check API Response

```bash
# Check if operation succeeded
slack-rs api call auth.test --raw | jq '.ok'
# Output: true or false

# Extract error message if failed
slack-rs api call chat.postMessage channel=INVALID text=Hello --raw | jq '.error'
# Output: "channel_not_found"
```

## Advanced Patterns

### Batch Operations

```bash
# Process multiple channels
export SLACKRS_OUTPUT=raw
for channel in C111 C222 C333; do
  echo "Processing $channel"
  slack-rs msg post $channel "Notification message"
done
```

### Safe Write Operations

```bash
# Script with write safety
set -e  # Exit on error

# Check write permission first
if [ "$SLACKCLI_ALLOW_WRITE" != "true" ]; then
  echo "Write operations disabled"
  exit 1
fi

# Perform write operation
slack-rs msg post C123456 "Deployment complete"
```

### Pipeline Processing

```bash
# Get channel list and process with jq
export SLACKRS_OUTPUT=raw
slack-rs api call conversations.list | \
  jq -r '.channels[] | select(.is_archived == false) | .id' | \
  while read channel; do
    echo "Active channel: $channel"
  done
```
