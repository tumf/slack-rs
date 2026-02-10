## MODIFIED Requirements

### Requirement: Provide export/import via CLI
Export/import of profiles MUST be executable from the CLI. `auth export --all` 実行時は、トークン未保存のプロファイルを警告してスキップし、他のプロファイルのエクスポートを継続しなければならない。(MUST)

#### Scenario: Export command encrypts and saves profile
- **WHEN** `slack-rs auth export --profile <name> --out <path>` is executed
- **THEN** the specified profile is encrypted and saved to the output path

#### Scenario: Export all skips profiles without tokens
- **WHEN** `slack-rs auth export --all --out <path>` is executed
- **AND** some profiles have no stored token
- **THEN** profiles with tokens are encrypted and saved to the output path
- **AND** profiles without tokens are reported as warnings and skipped

#### Scenario: Export all fails when no exportable profiles
- **WHEN** `slack-rs auth export --all --out <path>` is executed
- **AND** no profiles with tokens are available
- **THEN** the export operation is aborted with a clear error
