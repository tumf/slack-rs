# api-call Specification (Delta)

## ADDED Requirements

### Requirement: Send key=value as query params for GET requests
When `--get` is specified, `key=value` parameters MUST be sent as URL query parameters. GET requests MUST NOT send a request body. Even if `--json` is also specified, GET requests prioritize query parameters and do not send a JSON body. (MUST)

#### Scenario: Pass required parameters to conversations.replies with `--get`
- Given `--get` is specified
- And `channel=C123` and `ts=12345.6789` are specified as `key=value` parameters
- When executing `api call conversations.replies`
- Then the GET request query includes `channel=C123` and `ts=12345.6789`
- And no request body is sent
