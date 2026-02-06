## MODIFIED Requirements

### Requirement: Copy manifest to clipboard after generation

After the Manifest YAML is generated and saved during `auth login` execution, the clipboard copy operation MUST attempt multiple methods in the following order as a best-effort approach. If any method succeeds, the operation completes. If all methods fail, the process MUST NOT be interrupted.

- In SSH environments (TTY with `SSH_CONNECTION` or `SSH_TTY` present), OSC52 MUST be tried first.
- The `arboard` library MUST be tried.
- OS commands (macOS: `pbcopy` / Windows: `clip` / Linux: `wl-copy`, `xclip`, `xsel`) MUST be tried in order.

If the clipboard operation fails, a warning MUST be displayed with brief context and the process MUST NOT be interrupted.

#### Scenario: OSC52 is tried first in SSH environments
- Given executing `auth login` in an SSH environment
- When clipboard copy is performed after saving the manifest
- Then OSC52 copy is attempted first

#### Scenario: Fallback to OS commands when arboard fails
- Given `arboard` clipboard copy fails
- When the clipboard copy process continues
- Then OS commands are tried in sequence

#### Scenario: Clipboard is copied when available
- Given executing `auth login` and generating Manifest YAML
- When the manifest is saved to a file
- Then the same YAML is copied to the clipboard

#### Scenario: Continue with warning when clipboard is unavailable
- Given executing `auth login` in an environment where clipboard operations fail
- When attempting to copy to clipboard after saving the manifest
- Then a warning is displayed and the login process continues
