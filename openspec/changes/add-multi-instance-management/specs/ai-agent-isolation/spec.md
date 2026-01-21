## ADDED Requirements

> **Note**: In this specification, instance IDs like "inst-001" are simplified examples for readability. In the actual implementation, all instance IDs use UUID v4 format (e.g., "550e8400-e29b-41d4-a716-446655440000").

### Requirement: AI Agent Configuration Storage

Each instance MUST have isolated AI agent configurations.

#### Scenario: Store Claude Code configuration
- **WHEN** administrator configures Claude Code for instance "inst-001"
- **WITH** API key "sk-ant-xxx" and model "claude-sonnet-4-20250514"
- **THEN** system encrypts and stores the API key
- **AND** system stores configuration in `{data_root}/inst-001/ai-agents/claude-code/settings.json`
- **AND** configuration is not accessible to other instances

#### Scenario: Store Codex CLI configuration
- **WHEN** administrator configures Codex CLI for instance "inst-001"
- **WITH** API key "sk-xxx" and model "gpt-4"
- **THEN** system encrypts and stores the API key
- **AND** system stores configuration in `{data_root}/inst-001/ai-agents/codex-cli/config.yaml`

#### Scenario: Store Gemini CLI configuration
- **WHEN** administrator configures Gemini CLI for instance "inst-001"
- **WITH** API key "AIza-xxx" and model "gemini-pro"
- **THEN** system encrypts and stores the API key
- **AND** system stores configuration in `{data_root}/inst-001/ai-agents/gemini-cli/config.json`

#### Scenario: Store OpenCode configuration
- **WHEN** administrator configures OpenCode for instance "inst-001"
- **THEN** system stores configuration in `{data_root}/inst-001/ai-agents/opencode/config.toml`


### Requirement: AI Agent Environment Injection

When starting an instance, aicodex MUST inject AI agent environment variables.

#### Scenario: Inject Claude Code environment
- **WHEN** instance "inst-001" is started
- **AND** Claude Code is configured for the instance
- **THEN** system sets `ANTHROPIC_API_KEY` environment variable
- **AND** system sets `CLAUDE_CONFIG_DIR` to `{data_root}/inst-001/ai-agents/claude-code`

#### Scenario: Inject Codex CLI environment
- **WHEN** instance "inst-001" is started
- **AND** Codex CLI is configured for the instance
- **THEN** system sets `OPENAI_API_KEY` environment variable
- **AND** system sets `CODEX_CONFIG_HOME` to `{data_root}/inst-001/ai-agents/codex-cli`

#### Scenario: Inject Gemini CLI environment
- **WHEN** instance "inst-001" is started
- **AND** Gemini CLI is configured for the instance
- **THEN** system sets `GOOGLE_API_KEY` environment variable
- **AND** system sets `GEMINI_CONFIG_DIR` to `{data_root}/inst-001/ai-agents/gemini-cli`

#### Scenario: Inject OpenCode environment
- **WHEN** instance "inst-001" is started
- **AND** OpenCode is configured for the instance
- **THEN** system sets `OPENCODE_CONFIG_DIR` to `{data_root}/inst-001/ai-agents/opencode`

#### Scenario: No cross-instance environment leakage
- **WHEN** instance "inst-001" is running with Claude Code configured
- **AND** instance "inst-002" is running with different Claude Code configuration
- **THEN** instance "inst-001" cannot access instance "inst-002" API key
- **AND** instance "inst-002" cannot access instance "inst-001" API key


### Requirement: AI Agent Connection Testing

Administrators MUST be able to test AI agent connections.

#### Scenario: Test Claude Code connection
- **WHEN** administrator requests to test Claude Code for instance "inst-001"
- **THEN** system sends a minimal API request to Anthropic
- **AND** system returns success if response is valid
- **AND** system returns error message if connection fails

#### Scenario: Test Codex CLI connection
- **WHEN** administrator requests to test Codex CLI for instance "inst-001"
- **THEN** system sends a minimal API request to OpenAI
- **AND** system returns success if response is valid
- **AND** system returns error message if connection fails

#### Scenario: Test Gemini CLI connection
- **WHEN** administrator requests to test Gemini CLI for instance "inst-001"
- **THEN** system sends a minimal API request to Google AI
- **AND** system returns success if response is valid
- **AND** system returns error message if connection fails

#### Scenario: Test OpenCode connection
- **WHEN** administrator requests to test OpenCode for instance "inst-001"
- **THEN** system validates the configuration file exists
- **AND** system returns success if configuration is valid
- **AND** system returns error message if configuration is invalid


### Requirement: AI Agent Rate Limiting

Each instance SHALL have configurable rate limits for AI agents.

#### Scenario: Configure rate limit
- **WHEN** administrator sets rate limit of 60 requests per minute for Claude Code on instance "inst-001"
- **THEN** system stores the rate limit configuration
- **AND** system enforces the limit when instance makes API calls

#### Scenario: Rate limit exceeded
- **WHEN** instance "inst-001" exceeds the configured rate limit
- **THEN** system queues or rejects additional requests
- **AND** system logs the rate limit event

#### Scenario: Independent rate limits
- **WHEN** instance "inst-001" rate limit is 60 RPM
- **AND** instance "inst-002" rate limit is 120 RPM
- **THEN** each instance is limited independently
- **AND** one instance hitting limit does not affect the other


### Requirement: AI Agent Usage Tracking

aicodex MUST track AI agent usage per instance.

#### Scenario: Track API request count
- **WHEN** instance "inst-001" makes an API call to Claude Code
- **THEN** system increments the request count for the day
- **AND** request is associated with instance "inst-001"

#### Scenario: Track token usage
- **WHEN** instance "inst-001" receives a response from Claude Code
- **AND** response includes token usage metadata
- **THEN** system adds tokens to the daily count for instance "inst-001"

#### Scenario: Track errors
- **WHEN** instance "inst-001" API call to Claude Code fails
- **THEN** system increments the error count for the day
- **AND** system logs the error details

#### Scenario: Query usage statistics
- **WHEN** administrator requests usage statistics for instance "inst-001"
- **THEN** system returns daily aggregates for each AI agent
- **AND** includes: request_count, token_count, error_count


### Requirement: AI Agent Security

AI agent credentials MUST be stored securely.

#### Scenario: Encrypt API keys at rest
- **WHEN** administrator stores an API key for any AI agent
- **THEN** system encrypts the key using AES-256-GCM
- **AND** encryption key is derived from master secret
- **AND** plain text key is never written to disk

#### Scenario: Secure key retrieval
- **WHEN** instance is started and needs API key
- **THEN** system decrypts the key in memory
- **AND** passes key via environment variable
- **AND** key is not logged or exposed in error messages

#### Scenario: Key rotation
- **WHEN** administrator updates API key for an AI agent
- **THEN** system encrypts and stores the new key
- **AND** system marks the old key as rotated (for audit)
- **AND** running instance continues with old key until restart
