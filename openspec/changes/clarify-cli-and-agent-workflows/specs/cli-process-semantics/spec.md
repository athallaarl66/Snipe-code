## ADDED Requirements

### Requirement: Launcher and verification exports preserve stable process semantics
Launcher routing and verification report export SHALL use the existing process-code contract without treating a displayed menu, skipped action, or successful file write as proof that the selected operation passed.

#### Scenario: Non-interactive invocation lacks a command
- **WHEN** the executable receives no command outside an interactive terminal
- **THEN** it writes usage guidance, performs no operation, and exits with usage status `2`

#### Scenario: Interactive launcher exits before an operation
- **WHEN** the user selects Exit or cancels the launcher
- **THEN** the process exits with status `0` and changes no state

#### Scenario: Launcher-routed command finishes
- **WHEN** the launcher routes to analyze, audit, verify, or generation
- **THEN** the process uses the selected command's existing success, policy, usage, and runtime exit semantics

#### Scenario: Invalid verification report request
- **WHEN** verification receives an unsupported report format or lacks its required destination
- **THEN** it writes a usage error to standard error, runs no gate, and exits with status `2`

#### Scenario: Verification report cannot be written
- **WHEN** verification has a report result but cannot write the requested output file
- **THEN** it writes the export failure to standard error and exits with runtime status `3`

### Requirement: Interactive and machine-readable output channels remain separate
Human navigation and progress SHALL NOT be interleaved with requested structured output, and direct non-interactive commands SHALL NOT require launcher interaction.

#### Scenario: Direct JSON command
- **WHEN** a script invokes a direct command with JSON output
- **THEN** standard output contains parseable JSON and no launcher or wizard text

#### Scenario: Human-facing launcher
- **WHEN** the launcher is displayed interactively
- **THEN** it writes human-facing English text and does not claim to be structured output
