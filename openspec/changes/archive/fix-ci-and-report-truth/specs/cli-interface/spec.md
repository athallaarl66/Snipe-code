# Spec Delta

## Purpose

Defines the command surface of the CLI: how an invocation is routed to a command, what the tool reports about itself, and the exit status callers can script against.

## ADDED Requirements

### Requirement: CLI reports its own version
The CLI MUST expose its version through both a `version` command and a `--version` flag, printing the version value compiled into the binary.

#### Scenario: Version command
- **WHEN** the user runs `snipe-code version`
- **THEN** the CLI prints the compiled version and exits with status 0

#### Scenario: Version flag
- **WHEN** the user runs `snipe-code --version`
- **THEN** the CLI prints the same version string as the `version` command and exits with status 0

#### Scenario: Version output is machine-readable
- **WHEN** the version is printed
- **THEN** the output contains the version in `major.minor.patch` form on a single line

### Requirement: Unrecognized input is rejected as a usage error
An invocation that names neither a known command nor a known option MUST be rejected with a message naming the offending token and a usage-error exit status, without modifying the filesystem.

#### Scenario: Unknown command
- **WHEN** the user runs the CLI with an unrecognized command token
- **THEN** the CLI prints a message naming that token and exits with status 2

#### Scenario: Unknown option
- **WHEN** the user runs the CLI with an unrecognized option token
- **THEN** the CLI prints a message naming that token and exits with status 2
