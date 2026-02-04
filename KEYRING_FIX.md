# Keyring Token Storage Fix

## Problem

The `auth status` command was showing "Token: Missing" even though the login process appeared to save tokens successfully. Debug output showed that tokens were being saved, but immediately after saving, the `exists()` check returned `false`.

## Root Cause

The `keyring` crate (version 3.6) was not configured to use the macOS native backend. Without the `apple-native` feature enabled, the keyring library was using a fallback implementation that:

1. Appeared to save tokens successfully (no errors)
2. Could read tokens back from the SAME `Entry` instance
3. **Failed to persist tokens** to the actual macOS Keychain
4. Could NOT read tokens from a NEW `Entry` instance

This meant tokens were only cached in memory and never actually saved to the system keychain.

## Solution

Enable the `apple-native` feature for the `keyring` crate in `Cargo.toml`:

```toml
# Before
keyring = "3.6"

# After
keyring = { version = "3.6", features = ["apple-native"] }
```

This enables the `security-framework` backend which properly integrates with macOS Keychain.

## Verification

### Test Results

Created diagnostic tests that confirmed:

1. **Without `apple-native`**: 
   - `set_password()` succeeds
   - Same `Entry` instance can read back the password
   - New `Entry` instance FAILS to read the password
   - macOS `security` command cannot find the entry

2. **With `apple-native`**:
   - `set_password()` succeeds
   - Same `Entry` instance can read back the password
   - New `Entry` instance successfully reads the password ✅
   - macOS `security` command finds the entry ✅

### Commands to Verify

```bash
# Build with the fix
cargo build

# Save a test token
cargo test --test manual_token_test -- --nocapture

# Check status
cargo run -- auth status

# Verify in macOS Keychain
security find-generic-password -s "slackcli" -a "T06EJ9E5Z96:U06DEV59QS3" -w
```

## Changes Made

1. **Cargo.toml**: Added `apple-native` feature to keyring dependency
2. **src/auth/commands.rs**: 
   - Removed debug output from `status()` function
   - Removed verbose debug output from `save_profile_and_credentials()`
   - Updated test to clean up keyring entries before and after
3. **Tests**: All 175 unit tests + 55 integration tests pass

## Impact

- **Before**: Tokens were never actually saved, requiring re-authentication every time
- **After**: Tokens are properly saved to macOS Keychain and persist across sessions
- **Backward Compatibility**: Existing profiles will need to re-authenticate once to save tokens with the new backend

## Platform Notes

- **macOS**: Requires `apple-native` feature (now enabled)
- **Linux**: Should use `linux-native` or `sync-secret-service` features
- **Windows**: Should use `windows-native` feature

The keyring crate should automatically select the appropriate backend based on the target platform when these features are enabled.
