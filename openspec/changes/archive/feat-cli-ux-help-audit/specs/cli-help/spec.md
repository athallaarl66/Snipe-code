## ADDED Requirements

### Requirement: Help command overview
The system SHALL provide a `help` command and `--help` flag that displays usage: list of commands, options, and examples.

#### Scenario: Running --help
- **WHEN** user runs `snipe-code --help`
- **THEN** output lists available commands (`new` wizard, `audit`) and options (`--dry-run`, `--export`)

#### Scenario: Running help without arguments
- **WHEN** user runs `snipe-code help`
- **THEN** output shows the same usage overview as `--help`

### Requirement: Per-command help
The system SHALL provide detailed help per command via `snipe-code help <command>`.

#### Scenario: Help for audit command
- **WHEN** user runs `snipe-code help audit`
- **THEN** output explains the audit command, its options (`--export <md,pdf,docx>`), and shows a usage example

#### Scenario: Unknown help topic
- **WHEN** user runs `snipe-code help <unknown>`
- **THEN** CLI prints a message that the topic is unknown and points to `snipe-code help`

### Requirement: FAQ section in help
The system SHALL include a FAQ section in help output covering install, uninstall, and common troubleshooting.

#### Scenario: FAQ covers install problem
- **WHEN** user runs `snipe-code help` (or `snipe-code faq`)
- **THEN** output explains that `snipe-code` must be built/installed first (e.g. `cargo install --path .` or use release binary) to fix "'snipe-code' is not recognized"

#### Scenario: FAQ covers uninstall
- **WHEN** FAQ is displayed
- **THEN** output explains uninstall: `cargo uninstall snipe-code` when installed via cargo, or delete the binary file for portable `.exe`

### Requirement: Unknown command feedback
The system SHALL show a friendly error with a help hint when an unrecognized command is given.

#### Scenario: Unrecognized command
- **WHEN** user runs `snipe-code <unknown-command>`
- **THEN** CLI prints "Unknown command" with hint to run `snipe-code help`
